//! Finder delivers opened documents via `NSApplicationDelegate`, not argv.

use objc2::ffi;
use objc2::runtime::{AnyClass, AnyObject, Bool, Imp, Sel};
use objc2::sel;
use objc2_foundation::{NSArray, NSString, NSURL};
use std::ffi::CStr;
use std::mem;
use std::path::PathBuf;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::Mutex;

static PENDING: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());
static METHODS_HOOKED: AtomicBool = AtomicBool::new(false);
static DID_FINISH_SWIZZLED: AtomicBool = AtomicBool::new(false);
static ORIGINAL_DID_FINISH_LAUNCHING: AtomicPtr<()> = AtomicPtr::new(ptr::null_mut());

type DidFinishLaunchingFn =
    extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject);

pub fn take_pending_open_files() -> Vec<PathBuf> {
    PENDING.lock().unwrap().drain(..).collect()
}

pub fn open_handlers_installed() -> bool {
    METHODS_HOOKED.load(Ordering::Acquire)
}

fn push_path(path: String) {
    if path.is_empty() {
        return;
    }
    log::info!("macOS open document: {path}");
    PENDING.lock().unwrap().push(PathBuf::from(path));
}

fn push_nsstring(path: &NSString) {
    push_path(path.to_string());
}

fn push_urls(urls: &NSArray<NSURL>) {
    let count = urls.count();
    for i in 0..count {
        let url = urls.objectAtIndex(i);
        if url.isFileURL()
            && let Some(path) = url.path()
        {
            push_path(path.to_string());
        }
    }
}

/// `openFiles:` may deliver `NSString` paths or `NSURL` objects depending on macOS version.
fn push_open_files_list(files: &NSArray) {
    let count = files.count();
    for i in 0..count {
        let obj = files.objectAtIndex(i);
        if let Some(name) = obj.downcast_ref::<NSString>() {
            push_nsstring(name);
        } else if let Some(url) = obj.downcast_ref::<NSURL>()
            && url.isFileURL()
            && let Some(path) = url.path()
        {
            push_path(path.to_string());
        } else {
            log::warn!("macOS openFiles: unrecognized item at index {i}");
        }
    }
}

extern "C-unwind" fn application_open_files(
    _this: *mut AnyObject,
    _sel: Sel,
    _app: *mut AnyObject,
    files: *mut AnyObject,
) {
    let files: &NSArray = unsafe { &*files.cast() };
    push_open_files_list(files);
}

extern "C-unwind" fn application_open_file(
    _this: *mut AnyObject,
    _sel: Sel,
    _app: *mut AnyObject,
    file: *mut AnyObject,
) -> Bool {
    let file: &NSString = unsafe { &*file.cast() };
    push_nsstring(file);
    Bool::YES
}

extern "C-unwind" fn application_open_urls(
    _this: *mut AnyObject,
    _sel: Sel,
    _app: *mut AnyObject,
    urls: *mut AnyObject,
) {
    let urls: &NSArray<NSURL> = unsafe { &*urls.cast() };
    push_urls(urls);
}

extern "C-unwind" fn application_did_finish_launching_hook(
    this: *mut AnyObject,
    sel: Sel,
    notification: *mut AnyObject,
) {
    register_open_files_handler();
    let original = ORIGINAL_DID_FINISH_LAUNCHING.load(Ordering::Acquire);
    if !original.is_null() {
        let original: DidFinishLaunchingFn =
            unsafe { mem::transmute(original) };
        original(this, sel, notification);
    }
}

fn add_delegate_method(class: &AnyClass, sel: Sel, imp: Imp, types: &CStr) -> bool {
    let cls = ptr::from_ref(class).cast_mut();
    let added = unsafe { ffi::class_addMethod(cls, sel, imp, types.as_ptr()) };
    if added == Bool::NO {
        unsafe {
            let method = ffi::class_getInstanceMethod(cls, sel);
            if method.is_null() {
                return false;
            }
            ffi::method_setImplementation(method, imp);
        }
    }
    true
}

fn delegate_has_method(class: &AnyClass, sel: Sel) -> bool {
    let cls = ptr::from_ref(class).cast_mut();
    !unsafe { ffi::class_getInstanceMethod(cls, sel) }.is_null()
}

fn swizzle_did_finish_launching(class: &AnyClass) {
    if DID_FINISH_SWIZZLED.swap(true, Ordering::AcqRel) {
        return;
    }
    let cls = ptr::from_ref(class).cast_mut();
    let sel = sel!(applicationDidFinishLaunching:);
    let method = unsafe { ffi::class_getInstanceMethod(cls, sel) };
    if method.is_null() {
        log::warn!("WinitApplicationDelegate missing applicationDidFinishLaunching:");
        return;
    }
    let original = unsafe { ffi::method_getImplementation(method) };
    let original_ptr = original.map_or(ptr::null_mut(), |imp| imp as *mut ());
    ORIGINAL_DID_FINISH_LAUNCHING.store(original_ptr, Ordering::Release);
    let hook: Imp = unsafe { mem::transmute(application_did_finish_launching_hook as *const ()) };
    unsafe {
        ffi::method_setImplementation(method, hook);
    }
}

/// Installs `openFile` / `openFiles` on winit's delegate class. Safe to call repeatedly
/// until `WinitApplicationDelegate` exists (not loaded until the iced event loop starts).
pub fn register_open_files_handler() {
    if METHODS_HOOKED.load(Ordering::Acquire) {
        return;
    }
    let Some(class) = AnyClass::get(c"WinitApplicationDelegate") else {
        return;
    };

    swizzle_did_finish_launching(class);

    let open_files_imp: Imp = unsafe { mem::transmute(application_open_files as *const ()) };
    let open_file_imp: Imp = unsafe { mem::transmute(application_open_file as *const ()) };
    let open_urls_imp: Imp = unsafe { mem::transmute(application_open_urls as *const ()) };

    let ok = add_delegate_method(class, sel!(application:openFiles:), open_files_imp, c"v@:@@")
        && add_delegate_method(class, sel!(application:openFile:), open_file_imp, c"c@:@@")
        && add_delegate_method(class, sel!(application:openURLs:), open_urls_imp, c"v@:@@")
        && delegate_has_method(class, sel!(application:openFiles:));

    if ok {
        METHODS_HOOKED.store(true, Ordering::Release);
        log::info!("Installed Finder open-document handlers");
    } else {
        log::warn!("Failed to install Finder open-document handlers on WinitApplicationDelegate");
    }
}
