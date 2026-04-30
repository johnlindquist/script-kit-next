use std::{
    ffi::c_void,
    ptr,
    sync::mpsc,
    thread,
    time::Duration,
};

use accessibility_sys::{
    error_string, AXError as RawAxError, AXObserverRef, AXUIElementRef, AXValueRef, AXAPIEnabled,
    AXIsProcessTrusted, AXIsProcessTrustedWithOptions, AXObserverAddNotification, AXObserverCreate,
    AXObserverGetRunLoopSource, AXObserverRemoveNotification, AXUIElementCopyActionNames,
    AXUIElementCopyAttributeNames, AXUIElementCopyAttributeValue, AXUIElementCopyElementAtPosition,
    AXUIElementCopyParameterizedAttributeNames, AXUIElementCopyParameterizedAttributeValue,
    AXUIElementCreateApplication, AXUIElementCreateSystemWide, AXUIElementGetPid,
    AXUIElementGetTypeID, AXUIElementIsAttributeSettable, AXUIElementPerformAction,
    AXUIElementSetAttributeValue, AXUIElementSetMessagingTimeout, AXValueCreate, AXValueGetType,
    AXValueGetTypeID, AXValueGetValue, kAXErrorAPIDisabled, kAXErrorActionUnsupported,
    kAXErrorAttributeUnsupported, kAXErrorCannotComplete, kAXErrorFailure,
    kAXErrorIllegalArgument, kAXErrorInvalidUIElement, kAXErrorInvalidUIElementObserver,
    kAXErrorNoValue, kAXErrorNotEnoughPrecision, kAXErrorNotImplemented,
    kAXErrorNotificationAlreadyRegistered, kAXErrorNotificationNotRegistered,
    kAXErrorNotificationUnsupported, kAXErrorParameterizedAttributeUnsupported, kAXErrorSuccess,
    kAXTrustedCheckOptionPrompt, kAXValueTypeAXError, kAXValueTypeCFRange, kAXValueTypeCGPoint,
    kAXValueTypeCGRect, kAXValueTypeCGSize, kAXValueTypeIllegal,
};
use core_foundation::{
    base::TCFType,
    boolean::CFBoolean,
    dictionary::CFDictionary,
    string::CFString,
};
use core_foundation_sys::{
    array::{CFArrayGetCount, CFArrayGetTypeID, CFArrayGetValueAtIndex, CFArrayRef},
    base::{kCFAllocatorDefault, CFEqual, CFGetTypeID, CFRelease, CFRetain, CFTypeRef},
    dictionary::{CFDictionaryGetTypeID, CFDictionaryGetValue, CFDictionaryRef},
    number::{
        CFBooleanGetTypeID, CFBooleanGetValue, CFBooleanRef, CFNumberCreate, CFNumberGetTypeID,
        CFNumberGetValue, CFNumberRef, kCFBooleanFalse, kCFBooleanTrue, kCFNumberDoubleType,
        kCFNumberSInt64Type,
    },
    runloop::{
        kCFRunLoopDefaultMode, CFRunLoopAddSource, CFRunLoopGetCurrent, CFRunLoopRemoveSource,
        CFRunLoopRunInMode,
    },
    string::{CFStringGetTypeID, CFStringRef},
};

use crate::{
    attr, AxClientOptions, AxError, AxEvent, DisplayInfo, ElementSnapshot, PlatformValue, Point, Rect, Result,
    SettableValue, Size, TextRange, TreeOptions, WindowInfo, WindowQuery,
};

type CGWindowID = u32;
type CGWindowListOption = u32;
type CGEventRef = *mut c_void;
type CGDirectDisplayID = u32;
type CGError = i32;

const K_CG_ERROR_SUCCESS: CGError = 0;
const K_CG_NULL_WINDOW_ID: CGWindowID = 0;
const K_CG_WINDOW_LIST_OPTION_ALL: CGWindowListOption = 0;
const K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY: CGWindowListOption = 1;
const K_CG_WINDOW_LIST_EXCLUDE_DESKTOP_ELEMENTS: CGWindowListOption = 16;

const CG_WINDOW_NUMBER: &str = "kCGWindowNumber";
const CG_WINDOW_OWNER_PID: &str = "kCGWindowOwnerPID";
const CG_WINDOW_OWNER_NAME: &str = "kCGWindowOwnerName";
const CG_WINDOW_NAME: &str = "kCGWindowName";
const CG_WINDOW_BOUNDS: &str = "kCGWindowBounds";
const CG_WINDOW_LAYER: &str = "kCGWindowLayer";
const CG_WINDOW_ALPHA: &str = "kCGWindowAlpha";
const CG_WINDOW_IS_ONSCREEN: &str = "kCGWindowIsOnscreen";
const CG_WINDOW_SHARING_STATE: &str = "kCGWindowSharingState";
const CG_WINDOW_MEMORY_USAGE: &str = "kCGWindowMemoryUsage";

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGWindowListCopyWindowInfo(
        option: CGWindowListOption,
        relative_to_window: CGWindowID,
    ) -> CFArrayRef;
    fn CGRectMakeWithDictionaryRepresentation(
        dict: CFDictionaryRef,
        rect: *mut CGRectRepr,
    ) -> bool;
    fn CGEventCreate(source: *mut c_void) -> CGEventRef;
    fn CGEventGetLocation(event: CGEventRef) -> CGPointRepr;
    fn CGGetActiveDisplayList(
        max_displays: u32,
        active_displays: *mut CGDirectDisplayID,
        display_count: *mut u32,
    ) -> CGError;
    fn CGMainDisplayID() -> CGDirectDisplayID;
    fn CGDisplayBounds(display: CGDirectDisplayID) -> CGRectRepr;
}

#[derive(Debug)]
pub(crate) struct AxClientImpl {
    timeout: Option<Duration>,
}

impl Clone for AxClientImpl {
    fn clone(&self) -> Self {
        Self { timeout: self.timeout }
    }
}

#[derive(Debug)]
pub(crate) struct AxElementImpl {
    raw: AXUIElementRef,
}

impl Clone for AxElementImpl {
    fn clone(&self) -> Self {
        unsafe {
            CFRetain(self.raw as CFTypeRef);
        }
        Self { raw: self.raw }
    }
}

impl Drop for AxElementImpl {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                CFRelease(self.raw as CFTypeRef);
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct AxObserverImpl {
    stop_tx: mpsc::Sender<()>,
    thread: Option<thread::JoinHandle<()>>,
}

impl Drop for AxObserverImpl {
    fn drop(&mut self) {
        let _ = self.stop_tx.send(());
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

impl AxClientImpl {
    pub fn new(options: AxClientOptions) -> Result<Self> {
        if !Self::trusted(options.prompt_for_permission)? {
            return Err(AxError::NotTrusted);
        }

        unsafe {
            if !AXAPIEnabled() {
                return Err(AxError::ApiDisabled);
            }
        }

        Ok(Self {
            timeout: options.messaging_timeout,
        })
    }

    pub fn trusted(prompt: bool) -> Result<bool> {
        unsafe {
            if prompt {
                let key = CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt);
                let value = CFBoolean::wrap_under_get_rule(kCFBooleanTrue);
                let options = CFDictionary::from_CFType_pairs(&[(key, value)]);
                Ok(AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()))
            } else {
                Ok(AXIsProcessTrusted())
            }
        }
    }

    pub fn system_wide(&self) -> Result<AxElementImpl> {
        unsafe {
            let raw = AXUIElementCreateSystemWide();
            let element = AxElementImpl::from_create_rule(raw, "AXUIElementCreateSystemWide")?;
            self.apply_timeout(&element)?;
            Ok(element)
        }
    }

    pub fn application(&self, pid: i32) -> Result<AxElementImpl> {
        unsafe {
            let raw = AXUIElementCreateApplication(pid);
            let element = AxElementImpl::from_create_rule(raw, "AXUIElementCreateApplication")?;
            self.apply_timeout(&element)?;
            Ok(element)
        }
    }

    pub fn focused_application(&self) -> Result<AxElementImpl> {
        self.system_wide()?
            .element_attribute(attr::FOCUSED_APPLICATION)?
            .ok_or_else(|| AxError::NoValue {
                attribute: attr::FOCUSED_APPLICATION.to_string(),
            })
    }

    pub fn focused_element(&self) -> Result<AxElementImpl> {
        self.system_wide()?
            .element_attribute(attr::FOCUSED_UI_ELEMENT)?
            .ok_or_else(|| AxError::NoValue {
                attribute: attr::FOCUSED_UI_ELEMENT.to_string(),
            })
    }

    pub fn element_at_position(&self, point: Point) -> Result<AxElementImpl> {
        let system = self.system_wide()?;
        unsafe {
            let mut raw: AXUIElementRef = ptr::null_mut();
            let code = AXUIElementCopyElementAtPosition(
                system.raw,
                point.x as f32,
                point.y as f32,
                &mut raw,
            );
            map_ax_result(code, AxContext::Other("AXUIElementCopyElementAtPosition"))?;
            let element = AxElementImpl::from_create_rule(raw, "AXUIElementCopyElementAtPosition")?;
            self.apply_timeout(&element)?;
            Ok(element)
        }
    }

    pub fn mouse_location(&self) -> Result<Point> {
        unsafe {
            let event = CGEventCreate(ptr::null_mut());
            if event.is_null() {
                return Err(AxError::NullPointer("CGEventCreate"));
            }
            let point = CGEventGetLocation(event);
            CFRelease(event as CFTypeRef);
            Ok(Point::new(point.x, point.y))
        }
    }

    pub fn window_list(&self, query: WindowQuery) -> Result<Vec<WindowInfo>> {
        unsafe { copy_window_list(query) }
    }

    pub fn active_displays(&self) -> Result<Vec<DisplayInfo>> {
        unsafe { copy_active_displays() }
    }

    pub fn observe_application(
        &self,
        pid: i32,
        notifications: Vec<String>,
    ) -> Result<(AxObserverImpl, mpsc::Receiver<AxEvent>)> {
        let timeout = self.timeout;
        let (event_tx, event_rx) = mpsc::channel();
        let (stop_tx, stop_rx) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);

        let handle = thread::spawn(move || {
            run_observer_thread(pid, notifications, event_tx, stop_rx, timeout, ready_tx);
        });

        match ready_rx.recv() {
            Ok(Ok(())) => Ok((
                AxObserverImpl {
                    stop_tx,
                    thread: Some(handle),
                },
                event_rx,
            )),
            Ok(Err(error)) => {
                let _ = handle.join();
                Err(error)
            }
            Err(error) => {
                let _ = handle.join();
                Err(AxError::Observer(format!(
                    "observer thread exited before setup completed: {error}"
                )))
            }
        }
    }

    fn apply_timeout(&self, element: &AxElementImpl) -> Result<()> {
        if let Some(timeout) = self.timeout {
            unsafe {
                let code = AXUIElementSetMessagingTimeout(element.raw, timeout.as_secs_f32());
                map_ax_result(code, AxContext::Other("AXUIElementSetMessagingTimeout"))?;
            }
        }
        Ok(())
    }
}

impl AxElementImpl {
    unsafe fn from_create_rule(raw: AXUIElementRef, name: &'static str) -> Result<Self> {
        if raw.is_null() {
            Err(AxError::NullPointer(name))
        } else {
            Ok(Self { raw })
        }
    }

    unsafe fn from_borrowed(raw: AXUIElementRef, name: &'static str) -> Result<Self> {
        if raw.is_null() {
            return Err(AxError::NullPointer(name));
        }
        CFRetain(raw as CFTypeRef);
        Ok(Self { raw })
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        unsafe { self.raw == other.raw || CFEqual(self.raw as CFTypeRef, other.raw as CFTypeRef) != 0 }
    }

    pub fn pid(&self) -> Result<i32> {
        unsafe {
            let mut pid = 0;
            let code = AXUIElementGetPid(self.raw, &mut pid);
            map_ax_result(code, AxContext::Other("AXUIElementGetPid"))?;
            Ok(pid)
        }
    }

    pub fn attribute_names(&self) -> Result<Vec<String>> {
        unsafe {
            let mut names: CFArrayRef = ptr::null();
            let code = AXUIElementCopyAttributeNames(self.raw, &mut names);
            map_ax_result(code, AxContext::Other("AXUIElementCopyAttributeNames"))?;
            strings_from_created_array(names, "AXUIElementCopyAttributeNames")
        }
    }

    pub fn action_names(&self) -> Result<Vec<String>> {
        unsafe {
            let mut names: CFArrayRef = ptr::null();
            let code = AXUIElementCopyActionNames(self.raw, &mut names);
            map_ax_result(code, AxContext::Other("AXUIElementCopyActionNames"))?;
            strings_from_created_array(names, "AXUIElementCopyActionNames")
        }
    }

    pub fn parameterized_attribute_names(&self) -> Result<Vec<String>> {
        unsafe {
            let mut names: CFArrayRef = ptr::null();
            let code = AXUIElementCopyParameterizedAttributeNames(self.raw, &mut names);
            map_ax_result(
                code,
                AxContext::Other("AXUIElementCopyParameterizedAttributeNames"),
            )?;
            strings_from_created_array(names, "AXUIElementCopyParameterizedAttributeNames")
        }
    }

    pub fn attribute(&self, attribute: &str) -> Result<Option<PlatformValue>> {
        unsafe {
            let attr_name = CFString::new(attribute);
            let mut value: CFTypeRef = ptr::null();
            let code = AXUIElementCopyAttributeValue(
                self.raw,
                attr_name.as_concrete_TypeRef(),
                &mut value,
            );

            match code {
                kAXErrorSuccess => {
                    if value.is_null() {
                        Ok(None)
                    } else {
                        cf_type_to_value(value, Ownership::Create).map(Some)
                    }
                }
                kAXErrorNoValue => Ok(None),
                kAXErrorAttributeUnsupported | kAXErrorParameterizedAttributeUnsupported => Ok(None),
                _ => Err(map_ax_error(code, AxContext::Attribute(attribute))),
            }
        }
    }

    pub fn parameterized_attribute(
        &self,
        attribute: &str,
        parameter: SettableValue<'_>,
    ) -> Result<Option<PlatformValue>> {
        unsafe {
            let attr_name = CFString::new(attribute);
            let prepared = PreparedValue::new(parameter)?;
            let mut value: CFTypeRef = ptr::null();
            let code = AXUIElementCopyParameterizedAttributeValue(
                self.raw,
                attr_name.as_concrete_TypeRef(),
                prepared.as_cf_type_ref(),
                &mut value,
            );

            match code {
                kAXErrorSuccess => {
                    if value.is_null() {
                        Ok(None)
                    } else {
                        cf_type_to_value(value, Ownership::Create).map(Some)
                    }
                }
                kAXErrorNoValue => Ok(None),
                kAXErrorAttributeUnsupported | kAXErrorParameterizedAttributeUnsupported => Ok(None),
                _ => Err(map_ax_error(code, AxContext::Attribute(attribute))),
            }
        }
    }

    pub fn string_attribute(&self, attribute: &str) -> Result<Option<String>> {
        match self.attribute(attribute)? {
            Some(PlatformValue::String(value)) => Ok(Some(value)),
            Some(PlatformValue::Bool(value)) => Ok(Some(value.to_string())),
            Some(PlatformValue::I64(value)) => Ok(Some(value.to_string())),
            Some(PlatformValue::F64(value)) => Ok(Some(value.to_string())),
            Some(PlatformValue::Unsupported(value)) => Ok(Some(value)),
            Some(PlatformValue::Null) | None => Ok(None),
            Some(other) => Err(AxError::TypeMismatch {
                expected: "string-like AX value",
                actual: platform_value_kind(&other).to_string(),
            }),
        }
    }

    pub fn bool_attribute(&self, attribute: &str) -> Result<Option<bool>> {
        match self.attribute(attribute)? {
            Some(PlatformValue::Bool(value)) => Ok(Some(value)),
            Some(PlatformValue::Null) | None => Ok(None),
            Some(other) => Err(AxError::TypeMismatch {
                expected: "CFBoolean",
                actual: platform_value_kind(&other).to_string(),
            }),
        }
    }

    pub fn element_attribute(&self, attribute: &str) -> Result<Option<AxElementImpl>> {
        match self.attribute(attribute)? {
            Some(PlatformValue::Element(value)) => Ok(Some(value)),
            Some(PlatformValue::Null) | None => Ok(None),
            Some(other) => Err(AxError::TypeMismatch {
                expected: "AXUIElement",
                actual: platform_value_kind(&other).to_string(),
            }),
        }
    }

    pub fn children(&self) -> Result<Vec<AxElementImpl>> {
        match self.attribute(attr::CHILDREN)? {
            Some(PlatformValue::Elements(values)) => Ok(values),
            Some(PlatformValue::Element(value)) => Ok(vec![value]),
            Some(PlatformValue::Array(values)) => values
                .into_iter()
                .map(|value| match value {
                    PlatformValue::Element(element) => Ok(element),
                    other => Err(AxError::TypeMismatch {
                        expected: "array of AXUIElement",
                        actual: platform_value_kind(&other).to_string(),
                    }),
                })
                .collect(),
            Some(PlatformValue::Null) | None => Ok(Vec::new()),
            Some(other) => Err(AxError::TypeMismatch {
                expected: "array of AXUIElement",
                actual: platform_value_kind(&other).to_string(),
            }),
        }
    }

    pub fn frame(&self) -> Result<Option<Rect>> {
        let position = self.point_attribute(attr::POSITION)?;
        let size = self.size_attribute(attr::SIZE)?;
        Ok(match (position, size) {
            (Some(origin), Some(size)) => Some(Rect { origin, size }),
            _ => None,
        })
    }

    pub fn snapshot(&self, options: TreeOptions) -> Result<ElementSnapshot> {
        self.snapshot_inner(options, 0)
    }

    pub fn is_attribute_settable(&self, attribute: &str) -> Result<bool> {
        unsafe {
            let attr_name = CFString::new(attribute);
            let mut settable = 0;
            let code = AXUIElementIsAttributeSettable(
                self.raw,
                attr_name.as_concrete_TypeRef(),
                &mut settable,
            );
            match code {
                kAXErrorSuccess => Ok(settable != 0),
                kAXErrorAttributeUnsupported | kAXErrorNoValue => Ok(false),
                _ => Err(map_ax_error(code, AxContext::Attribute(attribute))),
            }
        }
    }

    pub fn set_attribute(&self, attribute: &str, value: SettableValue<'_>) -> Result<()> {
        unsafe {
            let attr_name = CFString::new(attribute);
            let prepared = PreparedValue::new(value)?;
            let code = AXUIElementSetAttributeValue(
                self.raw,
                attr_name.as_concrete_TypeRef(),
                prepared.as_cf_type_ref(),
            );
            map_ax_result(code, AxContext::Attribute(attribute))
        }
    }

    pub fn perform_action(&self, action: &str) -> Result<()> {
        unsafe {
            let action_name = CFString::new(action);
            let code = AXUIElementPerformAction(self.raw, action_name.as_concrete_TypeRef());
            map_ax_result(code, AxContext::Action(action))
        }
    }

    fn snapshot_inner(&self, options: TreeOptions, depth: usize) -> Result<ElementSnapshot> {
        let role = self.string_attribute(attr::ROLE).ok().flatten();
        let mut snapshot = ElementSnapshot {
            pid: self.pid().ok(),
            role: role.clone(),
            subrole: self.string_attribute(attr::SUBROLE).ok().flatten(),
            role_description: self.string_attribute(attr::ROLE_DESCRIPTION).ok().flatten(),
            title: self.string_attribute(attr::TITLE).ok().flatten(),
            value: self.string_attribute(attr::VALUE).ok().flatten(),
            description: self.string_attribute(attr::DESCRIPTION).ok().flatten(),
            identifier: self.string_attribute(attr::IDENTIFIER).ok().flatten(),
            enabled: self.bool_attribute(attr::ENABLED).ok().flatten(),
            focused: self.bool_attribute(attr::FOCUSED).ok().flatten(),
            selected: self.bool_attribute(attr::SELECTED).ok().flatten(),
            visible: self.bool_attribute(attr::VISIBLE).ok().flatten(),
            expanded: self.bool_attribute(attr::EXPANDED).ok().flatten(),
            main: self.bool_attribute(attr::MAIN).ok().flatten(),
            minimized: self.bool_attribute(attr::MINIMIZED).ok().flatten(),
            hidden: self.bool_attribute(attr::HIDDEN).ok().flatten(),
            frame: self.frame().ok().flatten(),
            children: Vec::new(),
        };

        if depth >= options.max_depth {
            return Ok(snapshot);
        }

        if !options.include_all_children && is_expensive_role(role.as_deref()) && depth > 0 {
            return Ok(snapshot);
        }

        let mut children = self.children().unwrap_or_default();
        if children.len() > options.max_children_per_node {
            children.truncate(options.max_children_per_node);
        }

        snapshot.children = children
            .into_iter()
            .filter_map(|child| child.snapshot_inner(options, depth + 1).ok())
            .collect();

        Ok(snapshot)
    }

    fn point_attribute(&self, attribute: &str) -> Result<Option<Point>> {
        match self.attribute(attribute)? {
            Some(PlatformValue::Point(value)) => Ok(Some(value)),
            Some(PlatformValue::Null) | None => Ok(None),
            Some(other) => Err(AxError::TypeMismatch {
                expected: "CGPoint AXValue",
                actual: platform_value_kind(&other).to_string(),
            }),
        }
    }

    fn size_attribute(&self, attribute: &str) -> Result<Option<Size>> {
        match self.attribute(attribute)? {
            Some(PlatformValue::Size(value)) => Ok(Some(value)),
            Some(PlatformValue::Null) | None => Ok(None),
            Some(other) => Err(AxError::TypeMismatch {
                expected: "CGSize AXValue",
                actual: platform_value_kind(&other).to_string(),
            }),
        }
    }
}

fn run_observer_thread(
    pid: i32,
    notifications: Vec<String>,
    event_tx: mpsc::Sender<AxEvent>,
    stop_rx: mpsc::Receiver<()>,
    timeout: Option<Duration>,
    ready_tx: mpsc::SyncSender<Result<()>>,
) {
    unsafe {
        let app = match AxElementImpl::from_create_rule(
            AXUIElementCreateApplication(pid),
            "AXUIElementCreateApplication",
        ) {
            Ok(app) => app,
            Err(error) => {
                let _ = ready_tx.send(Err(error));
                return;
            }
        };
        if let Some(timeout) = timeout {
            let _ = AXUIElementSetMessagingTimeout(app.raw, timeout.as_secs_f32());
        }

        let mut observer: AXObserverRef = ptr::null_mut();
        let code = AXObserverCreate(pid, observer_callback, &mut observer);
        if let Err(error) = map_ax_result(code, AxContext::Other("AXObserverCreate")) {
            let _ = ready_tx.send(Err(error));
            return;
        }
        if observer.is_null() {
            let _ = ready_tx.send(Err(AxError::NullPointer("AXObserverCreate")));
            return;
        }

        let source = AXObserverGetRunLoopSource(observer);
        if source.is_null() {
            CFRelease(observer as CFTypeRef);
            let _ = ready_tx.send(Err(AxError::NullPointer("AXObserverGetRunLoopSource")));
            return;
        }

        let state = Box::new(ObserverState { tx: event_tx, pid });
        let state_ptr = Box::into_raw(state);
        let current_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(current_loop, source, kCFRunLoopDefaultMode);

        let setup_result = (|| -> Result<()> {
            for notification in &notifications {
                let notification_name = CFString::new(notification);
                let code = AXObserverAddNotification(
                    observer,
                    app.raw,
                    notification_name.as_concrete_TypeRef(),
                    state_ptr as *mut c_void,
                );
                match code {
                    kAXErrorSuccess | kAXErrorNotificationAlreadyRegistered => {}
                    _ => return Err(map_ax_error(code, AxContext::Notification(notification))),
                }
            }
            Ok(())
        })();

        if let Err(error) = setup_result {
            CFRunLoopRemoveSource(current_loop, source, kCFRunLoopDefaultMode);
            let _ = Box::from_raw(state_ptr);
            CFRelease(observer as CFTypeRef);
            let _ = ready_tx.send(Err(error));
            return;
        }

        let _ = ready_tx.send(Ok(()));

        while stop_rx.try_recv().is_err() {
            let _ = CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.1, 1);
        }

        for notification in &notifications {
            let notification_name = CFString::new(notification);
            let _ = AXObserverRemoveNotification(
                observer,
                app.raw,
                notification_name.as_concrete_TypeRef(),
            );
        }
        CFRunLoopRemoveSource(current_loop, source, kCFRunLoopDefaultMode);
        let _ = Box::from_raw(state_ptr);
        CFRelease(observer as CFTypeRef);
    }
}

struct ObserverState {
    tx: mpsc::Sender<AxEvent>,
    pid: i32,
}

unsafe extern "C" fn observer_callback(
    _observer: AXObserverRef,
    element: AXUIElementRef,
    notification: CFStringRef,
    refcon: *mut c_void,
) {
    if refcon.is_null() {
        return;
    }

    let state = &*(refcon as *const ObserverState);
    let notification = cf_string_to_string(notification);
    let element = if element.is_null() {
        None
    } else {
        AxElementImpl::from_borrowed(element, "AXObserver callback element")
            .ok()
            .and_then(|element| {
                element
                    .snapshot(TreeOptions {
                        max_depth: 2,
                        max_children_per_node: 32,
                        include_all_children: false,
                    })
                    .ok()
            })
    };

    let _ = state.tx.send(AxEvent {
        pid: state.pid,
        notification,
        element,
    });
}

unsafe fn copy_window_list(query: WindowQuery) -> Result<Vec<WindowInfo>> {
    let mut options = if query.on_screen_only {
        K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY
    } else {
        K_CG_WINDOW_LIST_OPTION_ALL
    };
    if !query.include_desktop_elements {
        options |= K_CG_WINDOW_LIST_EXCLUDE_DESKTOP_ELEMENTS;
    }

    let array = CGWindowListCopyWindowInfo(options, K_CG_NULL_WINDOW_ID);
    if array.is_null() {
        return Err(AxError::NullPointer("CGWindowListCopyWindowInfo"));
    }

    let result = (|| -> Result<Vec<WindowInfo>> {
        let count = CFArrayGetCount(array);
        let mut windows = Vec::with_capacity(count as usize);
        for index in 0..count {
            let value = CFArrayGetValueAtIndex(array, index) as CFTypeRef;
            if value.is_null() || CFGetTypeID(value) != CFDictionaryGetTypeID() {
                continue;
            }
            if let Some(info) = window_info_from_dictionary(value as CFDictionaryRef)? {
                if query.matches(&info) {
                    windows.push(info);
                }
            }
        }
        Ok(windows)
    })();

    CFRelease(array as CFTypeRef);
    result
}

unsafe fn copy_active_displays() -> Result<Vec<DisplayInfo>> {
    let mut count = 0_u32;
    let code = CGGetActiveDisplayList(0, ptr::null_mut(), &mut count);
    if code != K_CG_ERROR_SUCCESS {
        return Err(AxError::WindowList(format!("CGGetActiveDisplayList failed with CGError {code}")));
    }
    if count == 0 {
        return Ok(Vec::new());
    }

    let mut ids = vec![0_u32; count as usize];
    let code = CGGetActiveDisplayList(count, ids.as_mut_ptr(), &mut count);
    if code != K_CG_ERROR_SUCCESS {
        return Err(AxError::WindowList(format!("CGGetActiveDisplayList failed with CGError {code}")));
    }
    ids.truncate(count as usize);

    let main = CGMainDisplayID();
    Ok(ids
        .into_iter()
        .map(|id| {
            let raw = CGDisplayBounds(id);
            DisplayInfo {
                id,
                bounds: Rect {
                    origin: Point::new(raw.origin.x, raw.origin.y),
                    size: Size::new(raw.size.width, raw.size.height),
                },
                is_main: id == main,
            }
        })
        .collect())
}

unsafe fn window_info_from_dictionary(dict: CFDictionaryRef) -> Result<Option<WindowInfo>> {
    let Some(id) = dict_get_i64(dict, CG_WINDOW_NUMBER)? else {
        return Ok(None);
    };
    let Some(owner_pid) = dict_get_i64(dict, CG_WINDOW_OWNER_PID)? else {
        return Ok(None);
    };
    let Some(bounds) = dict_get_rect(dict, CG_WINDOW_BOUNDS)? else {
        return Ok(None);
    };

    Ok(Some(WindowInfo {
        id: id as u32,
        owner_pid: owner_pid as i32,
        owner_name: dict_get_string(dict, CG_WINDOW_OWNER_NAME)?,
        title: dict_get_string(dict, CG_WINDOW_NAME)?,
        bounds,
        layer: dict_get_i64(dict, CG_WINDOW_LAYER)?.unwrap_or_default(),
        alpha: dict_get_f64(dict, CG_WINDOW_ALPHA)?,
        is_on_screen: dict_get_bool(dict, CG_WINDOW_IS_ONSCREEN)?,
        sharing_state: dict_get_i64(dict, CG_WINDOW_SHARING_STATE)?,
        memory_usage: dict_get_i64(dict, CG_WINDOW_MEMORY_USAGE)?,
    }))
}

unsafe fn dict_get_value(dict: CFDictionaryRef, key: &str) -> Option<CFTypeRef> {
    let key = CFString::new(key);
    let value = CFDictionaryGetValue(dict, key.as_concrete_TypeRef() as *const c_void) as CFTypeRef;
    if value.is_null() {
        None
    } else {
        Some(value)
    }
}

unsafe fn dict_get_string(dict: CFDictionaryRef, key: &str) -> Result<Option<String>> {
    let Some(value) = dict_get_value(dict, key) else {
        return Ok(None);
    };
    if CFGetTypeID(value) == CFStringGetTypeID() {
        Ok(Some(cf_string_to_string(value as CFStringRef)))
    } else {
        Err(AxError::TypeMismatch {
            expected: "CFString",
            actual: format!("CFTypeID({})", CFGetTypeID(value)),
        })
    }
}

unsafe fn dict_get_i64(dict: CFDictionaryRef, key: &str) -> Result<Option<i64>> {
    let Some(value) = dict_get_value(dict, key) else {
        return Ok(None);
    };
    if CFGetTypeID(value) == CFNumberGetTypeID() {
        Ok(Some(cf_number_to_i64(value as CFNumberRef)?))
    } else {
        Err(AxError::TypeMismatch {
            expected: "CFNumber",
            actual: format!("CFTypeID({})", CFGetTypeID(value)),
        })
    }
}

unsafe fn dict_get_f64(dict: CFDictionaryRef, key: &str) -> Result<Option<f64>> {
    let Some(value) = dict_get_value(dict, key) else {
        return Ok(None);
    };
    if CFGetTypeID(value) == CFNumberGetTypeID() {
        Ok(Some(cf_number_to_f64(value as CFNumberRef)?))
    } else {
        Err(AxError::TypeMismatch {
            expected: "CFNumber",
            actual: format!("CFTypeID({})", CFGetTypeID(value)),
        })
    }
}

unsafe fn dict_get_bool(dict: CFDictionaryRef, key: &str) -> Result<Option<bool>> {
    let Some(value) = dict_get_value(dict, key) else {
        return Ok(None);
    };
    if CFGetTypeID(value) == CFBooleanGetTypeID() {
        Ok(Some(CFBooleanGetValue(value as CFBooleanRef)))
    } else if CFGetTypeID(value) == CFNumberGetTypeID() {
        Ok(Some(cf_number_to_i64(value as CFNumberRef)? != 0))
    } else {
        Err(AxError::TypeMismatch {
            expected: "CFBoolean or CFNumber",
            actual: format!("CFTypeID({})", CFGetTypeID(value)),
        })
    }
}

unsafe fn dict_get_rect(dict: CFDictionaryRef, key: &str) -> Result<Option<Rect>> {
    let Some(value) = dict_get_value(dict, key) else {
        return Ok(None);
    };
    if CFGetTypeID(value) != CFDictionaryGetTypeID() {
        return Err(AxError::TypeMismatch {
            expected: "CFDictionary CGRect representation",
            actual: format!("CFTypeID({})", CFGetTypeID(value)),
        });
    }

    let mut raw = CGRectRepr::default();
    if CGRectMakeWithDictionaryRepresentation(value as CFDictionaryRef, &mut raw) {
        Ok(Some(Rect {
            origin: Point::new(raw.origin.x, raw.origin.y),
            size: Size::new(raw.size.width, raw.size.height),
        }))
    } else {
        Err(AxError::WindowList(format!(
            "{key} could not be decoded as a CGRect"
        )))
    }
}

#[derive(Clone, Copy)]
enum Ownership {
    Create,
    Borrow,
}

unsafe fn cf_type_to_value(value: CFTypeRef, ownership: Ownership) -> Result<PlatformValue> {
    if value.is_null() {
        return Ok(PlatformValue::Null);
    }

    let type_id = CFGetTypeID(value);
    let result = if type_id == AXUIElementGetTypeID() {
        let element = match ownership {
            Ownership::Create => AxElementImpl::from_create_rule(value as AXUIElementRef, "AXUIElement")?,
            Ownership::Borrow => AxElementImpl::from_borrowed(value as AXUIElementRef, "AXUIElement")?,
        };
        return Ok(PlatformValue::Element(element));
    } else if type_id == CFStringGetTypeID() {
        PlatformValue::String(cf_string_to_string(value as CFStringRef))
    } else if type_id == CFBooleanGetTypeID() {
        PlatformValue::Bool(CFBooleanGetValue(value as CFBooleanRef))
    } else if type_id == CFNumberGetTypeID() {
        PlatformValue::I64(cf_number_to_i64(value as CFNumberRef)?)
    } else if type_id == AXValueGetTypeID() {
        ax_value_to_value(value as AXValueRef)?
    } else if type_id == CFArrayGetTypeID() {
        cf_array_to_value(value as CFArrayRef)?
    } else {
        PlatformValue::Unsupported(format!("CFTypeID({type_id})"))
    };

    if matches!(ownership, Ownership::Create) {
        CFRelease(value);
    }

    Ok(result)
}

unsafe fn cf_array_to_value(array: CFArrayRef) -> Result<PlatformValue> {
    if array.is_null() {
        return Ok(PlatformValue::Null);
    }

    let count = CFArrayGetCount(array);
    let mut values = Vec::with_capacity(count as usize);
    for index in 0..count {
        let item = CFArrayGetValueAtIndex(array, index) as CFTypeRef;
        values.push(cf_type_to_value(item, Ownership::Borrow)?);
    }

    if values.iter().all(|value| matches!(value, PlatformValue::Element(_))) {
        let elements = values
            .into_iter()
            .map(|value| match value {
                PlatformValue::Element(element) => element,
                _ => unreachable!(),
            })
            .collect();
        Ok(PlatformValue::Elements(elements))
    } else {
        Ok(PlatformValue::Array(values))
    }
}

unsafe fn strings_from_created_array(array: CFArrayRef, source: &'static str) -> Result<Vec<String>> {
    if array.is_null() {
        return Err(AxError::NullPointer(source));
    }

    let result = (|| {
        let count = CFArrayGetCount(array);
        let mut values = Vec::with_capacity(count as usize);
        for index in 0..count {
            let item = CFArrayGetValueAtIndex(array, index) as CFTypeRef;
            if item.is_null() || CFGetTypeID(item) != CFStringGetTypeID() {
                return Err(AxError::TypeMismatch {
                    expected: "array of CFString",
                    actual: if item.is_null() {
                        "null".to_string()
                    } else {
                        format!("CFTypeID({})", CFGetTypeID(item))
                    },
                });
            }
            values.push(cf_string_to_string(item as CFStringRef));
        }
        Ok(values)
    })();

    CFRelease(array as CFTypeRef);
    result
}

unsafe fn cf_string_to_string(value: CFStringRef) -> String {
    CFString::wrap_under_get_rule(value).to_string()
}

unsafe fn cf_number_to_i64(number: CFNumberRef) -> Result<i64> {
    let mut int_value = 0_i64;
    if CFNumberGetValue(
        number,
        kCFNumberSInt64Type,
        &mut int_value as *mut i64 as *mut c_void,
    ) {
        Ok(int_value)
    } else {
        let mut float_value = 0_f64;
        if CFNumberGetValue(
            number,
            kCFNumberDoubleType,
            &mut float_value as *mut f64 as *mut c_void,
        ) {
            Ok(float_value as i64)
        } else {
            Err(AxError::TypeMismatch {
                expected: "CFNumber",
                actual: "unreadable CFNumber".to_string(),
            })
        }
    }
}

unsafe fn cf_number_to_f64(number: CFNumberRef) -> Result<f64> {
    let mut float_value = 0_f64;
    if CFNumberGetValue(
        number,
        kCFNumberDoubleType,
        &mut float_value as *mut f64 as *mut c_void,
    ) {
        Ok(float_value)
    } else {
        let mut int_value = 0_i64;
        if CFNumberGetValue(
            number,
            kCFNumberSInt64Type,
            &mut int_value as *mut i64 as *mut c_void,
        ) {
            Ok(int_value as f64)
        } else {
            Err(AxError::TypeMismatch {
                expected: "CFNumber",
                actual: "unreadable CFNumber".to_string(),
            })
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct CGPointRepr {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct CGSizeRepr {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct CGRectRepr {
    origin: CGPointRepr,
    size: CGSizeRepr,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct CFRangeRepr {
    location: isize,
    length: isize,
}

unsafe fn ax_value_to_value(value: AXValueRef) -> Result<PlatformValue> {
    let value_type = AXValueGetType(value);
    match value_type {
        kAXValueTypeCGPoint => {
            let mut raw = CGPointRepr::default();
            read_ax_value(value, value_type, &mut raw)?;
            Ok(PlatformValue::Point(Point { x: raw.x, y: raw.y }))
        }
        kAXValueTypeCGSize => {
            let mut raw = CGSizeRepr::default();
            read_ax_value(value, value_type, &mut raw)?;
            Ok(PlatformValue::Size(Size {
                width: raw.width,
                height: raw.height,
            }))
        }
        kAXValueTypeCGRect => {
            let mut raw = CGRectRepr::default();
            read_ax_value(value, value_type, &mut raw)?;
            Ok(PlatformValue::Rect(Rect {
                origin: Point {
                    x: raw.origin.x,
                    y: raw.origin.y,
                },
                size: Size {
                    width: raw.size.width,
                    height: raw.size.height,
                },
            }))
        }
        kAXValueTypeCFRange => {
            let mut raw = CFRangeRepr::default();
            read_ax_value(value, value_type, &mut raw)?;
            Ok(PlatformValue::Range(TextRange {
                location: raw.location as i64,
                length: raw.length as i64,
            }))
        }
        kAXValueTypeAXError => {
            let mut raw = 0_i32;
            read_ax_value(value, value_type, &mut raw)?;
            Ok(PlatformValue::Unsupported(format!(
                "embedded {} ({raw})",
                error_string(raw)
            )))
        }
        kAXValueTypeIllegal => Ok(PlatformValue::Unsupported("illegal AXValue".to_string())),
        other => Ok(PlatformValue::Unsupported(format!("AXValueType({other})"))),
    }
}

unsafe fn read_ax_value<T>(value: AXValueRef, value_type: u32, out: &mut T) -> Result<()> {
    if AXValueGetValue(value, value_type, out as *mut T as *mut c_void) {
        Ok(())
    } else {
        Err(AxError::TypeMismatch {
            expected: "AXValue",
            actual: format!("AXValueType({value_type})"),
        })
    }
}

enum PreparedValue {
    String(CFString),
    Number(CFNumberRef),
    AxValue(AXValueRef),
    Borrowed(CFTypeRef),
}

impl PreparedValue {
    unsafe fn new(value: SettableValue<'_>) -> Result<Self> {
        match value {
            SettableValue::String(value) => Ok(Self::String(CFString::new(value))),
            SettableValue::Bool(value) => Ok(Self::Borrowed(if value {
                kCFBooleanTrue as CFTypeRef
            } else {
                kCFBooleanFalse as CFTypeRef
            })),
            SettableValue::I64(value) => {
                let number = CFNumberCreate(
                    kCFAllocatorDefault,
                    kCFNumberSInt64Type,
                    &value as *const i64 as *const c_void,
                );
                if number.is_null() {
                    Err(AxError::NullPointer("CFNumberCreate"))
                } else {
                    Ok(Self::Number(number))
                }
            }
            SettableValue::F64(value) => {
                let number = CFNumberCreate(
                    kCFAllocatorDefault,
                    kCFNumberDoubleType,
                    &value as *const f64 as *const c_void,
                );
                if number.is_null() {
                    Err(AxError::NullPointer("CFNumberCreate"))
                } else {
                    Ok(Self::Number(number))
                }
            }
            SettableValue::Point(point) => {
                let raw = CGPointRepr { x: point.x, y: point.y };
                Self::new_ax_value(kAXValueTypeCGPoint, &raw)
            }
            SettableValue::Size(size) => {
                let raw = CGSizeRepr {
                    width: size.width,
                    height: size.height,
                };
                Self::new_ax_value(kAXValueTypeCGSize, &raw)
            }
            SettableValue::Rect(rect) => {
                let raw = CGRectRepr {
                    origin: CGPointRepr {
                        x: rect.origin.x,
                        y: rect.origin.y,
                    },
                    size: CGSizeRepr {
                        width: rect.size.width,
                        height: rect.size.height,
                    },
                };
                Self::new_ax_value(kAXValueTypeCGRect, &raw)
            }
            SettableValue::Range(range) => {
                let raw = CFRangeRepr {
                    location: range.location as isize,
                    length: range.length as isize,
                };
                Self::new_ax_value(kAXValueTypeCFRange, &raw)
            }
            SettableValue::Element(element) => Ok(Self::Borrowed(element.0.raw as CFTypeRef)),
        }
    }

    unsafe fn new_ax_value<T>(value_type: u32, raw: &T) -> Result<Self> {
        let value = AXValueCreate(value_type, raw as *const T as *const c_void);
        if value.is_null() {
            Err(AxError::NullPointer("AXValueCreate"))
        } else {
            Ok(Self::AxValue(value))
        }
    }

    fn as_cf_type_ref(&self) -> CFTypeRef {
        match self {
            PreparedValue::String(value) => value.as_CFTypeRef(),
            PreparedValue::Number(value) => *value as CFTypeRef,
            PreparedValue::AxValue(value) => *value as CFTypeRef,
            PreparedValue::Borrowed(value) => *value,
        }
    }
}

impl Drop for PreparedValue {
    fn drop(&mut self) {
        unsafe {
            match self {
                PreparedValue::Number(value) => CFRelease(*value as CFTypeRef),
                PreparedValue::AxValue(value) => CFRelease(*value as CFTypeRef),
                PreparedValue::String(_) | PreparedValue::Borrowed(_) => {}
            }
        }
    }
}

enum AxContext<'a> {
    Attribute(&'a str),
    Action(&'a str),
    Notification(&'a str),
    Other(&'static str),
}

fn map_ax_result(code: RawAxError, context: AxContext<'_>) -> Result<()> {
    if code == kAXErrorSuccess {
        Ok(())
    } else {
        Err(map_ax_error(code, context))
    }
}

fn map_ax_error(code: RawAxError, context: AxContext<'_>) -> AxError {
    match code {
        kAXErrorAPIDisabled => AxError::ApiDisabled,
        kAXErrorInvalidUIElement | kAXErrorInvalidUIElementObserver => AxError::InvalidElement,
        kAXErrorCannotComplete => AxError::CannotComplete,
        kAXErrorAttributeUnsupported | kAXErrorParameterizedAttributeUnsupported => match context {
            AxContext::Attribute(attribute) => AxError::AttributeUnsupported {
                attribute: attribute.to_string(),
            },
            _ => AxError::Ax {
                code,
                message: error_string(code).to_string(),
            },
        },
        kAXErrorNoValue => match context {
            AxContext::Attribute(attribute) => AxError::NoValue {
                attribute: attribute.to_string(),
            },
            _ => AxError::Ax {
                code,
                message: error_string(code).to_string(),
            },
        },
        kAXErrorActionUnsupported => match context {
            AxContext::Action(action) => AxError::ActionUnsupported {
                action: action.to_string(),
            },
            _ => AxError::Ax {
                code,
                message: error_string(code).to_string(),
            },
        },
        kAXErrorNotificationUnsupported => match context {
            AxContext::Notification(notification) => AxError::NotificationUnsupported {
                notification: notification.to_string(),
            },
            _ => AxError::Ax {
                code,
                message: error_string(code).to_string(),
            },
        },
        kAXErrorFailure | kAXErrorIllegalArgument | kAXErrorNotImplemented
        | kAXErrorNotificationAlreadyRegistered | kAXErrorNotificationNotRegistered
        | kAXErrorNotEnoughPrecision => AxError::Ax {
            code,
            message: match context {
                AxContext::Other(name) => format!("{} in {name}", error_string(code)),
                _ => error_string(code).to_string(),
            },
        },
        _ => AxError::Ax {
            code,
            message: match context {
                AxContext::Other(name) => format!("unknown AX error in {name}"),
                _ => "unknown AX error".to_string(),
            },
        },
    }
}

fn platform_value_kind(value: &PlatformValue) -> &'static str {
    match value {
        PlatformValue::String(_) => "CFString",
        PlatformValue::Bool(_) => "CFBoolean",
        PlatformValue::I64(_) => "CFNumber",
        PlatformValue::F64(_) => "CFNumber",
        PlatformValue::Element(_) => "AXUIElement",
        PlatformValue::Elements(_) => "array of AXUIElement",
        PlatformValue::Point(_) => "CGPoint AXValue",
        PlatformValue::Size(_) => "CGSize AXValue",
        PlatformValue::Rect(_) => "CGRect AXValue",
        PlatformValue::Range(_) => "CFRange AXValue",
        PlatformValue::Array(_) => "CFArray",
        PlatformValue::Null => "null",
        PlatformValue::Unsupported(_) => "unsupported CFType",
    }
}

fn is_expensive_role(role: Option<&str>) -> bool {
    matches!(
        role,
        Some("AXTable" | "AXOutline" | "AXBrowser" | "AXWebArea" | "AXScrollArea")
    )
}
