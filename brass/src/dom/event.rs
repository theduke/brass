use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

pub trait DomEvent: Sized {
    fn event_type() -> crate::dom::Event;
    fn from_dom(ev: web_sys::Event) -> Option<Self>;
}

impl DomEvent for web_sys::InputEvent {
    fn event_type() -> crate::dom::Event {
        crate::dom::Event::Input
    }

    fn from_dom(ev: web_sys::Event) -> Option<Self> {
        ev.dyn_into().ok()
    }
}

// InputEvent

pub struct InputEvent(pub web_sys::InputEvent);

impl InputEvent {
    pub fn value(&self) -> Option<String> {
        let target = self.0.current_target()?;

        // TODO perf: can these 3 checks be reduced to just one?
        if let Some(input) = target.dyn_ref::<web_sys::HtmlInputElement>() {
            Some(input.value())
        } else if let Some(textarea) = target.dyn_ref::<web_sys::HtmlTextAreaElement>() {
            Some(textarea.value())
        } else if let Some(select) = target.dyn_ref::<web_sys::HtmlSelectElement>() {
            Some(select.value())
        } else {
            None
        }
    }
}

impl std::ops::Deref for InputEvent {
    type Target = web_sys::InputEvent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DomEvent for InputEvent {
    fn event_type() -> crate::dom::Event {
        crate::dom::Event::Input
    }

    fn from_dom(ev: web_sys::Event) -> Option<Self> {
        ev.dyn_into().ok().map(Self)
    }
}

// ChangeEvent

pub struct ChangeEvent(pub web_sys::Event);

impl ChangeEvent {
    pub fn value(&self) -> Option<String> {
        let target = self.0.current_target()?;

        // TODO perf: can these 3 checks be reduced to just one?
        if let Some(input) = target.dyn_ref::<web_sys::HtmlInputElement>() {
            Some(input.value())
        } else if let Some(textarea) = target.dyn_ref::<web_sys::HtmlTextAreaElement>() {
            Some(textarea.value())
        } else if let Some(select) = target.dyn_ref::<web_sys::HtmlSelectElement>() {
            Some(select.value())
        } else {
            None
        }
    }
}

impl std::ops::Deref for ChangeEvent {
    type Target = web_sys::Event;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DomEvent for ChangeEvent {
    fn event_type() -> crate::dom::Event {
        crate::dom::Event::Change
    }

    fn from_dom(ev: web_sys::Event) -> Option<Self> {
        ev.dyn_into().ok().map(Self)
    }
}

// ClickEvent.

pub struct ClickEvent(pub web_sys::MouseEvent);

impl std::ops::Deref for ClickEvent {
    type Target = web_sys::MouseEvent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DomEvent for ClickEvent {
    fn event_type() -> crate::dom::Event {
        crate::dom::Event::Click
    }

    fn from_dom(ev: web_sys::Event) -> Option<Self> {
        ev.dyn_into().ok().map(Self)
    }
}

// CheckboxInputEvent.

pub struct CheckboxInputEvent(pub web_sys::InputEvent);

impl CheckboxInputEvent {
    pub fn value(&self) -> Option<bool> {
        let flag = self
            .0
            .current_target()?
            .dyn_ref::<HtmlInputElement>()?
            .checked();
        Some(flag)
    }
}

// impl std::ops::Deref for ClickEvent {
//     type Target = web_sys::MouseEvent;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

impl DomEvent for CheckboxInputEvent {
    fn event_type() -> crate::dom::Event {
        crate::dom::Event::Change
    }

    fn from_dom(ev: web_sys::Event) -> Option<Self> {
        ev.dyn_into().ok().map(Self)
    }
}

// KeyDownEvent.

pub struct KeyDownEvent(pub web_sys::KeyboardEvent);

impl std::ops::Deref for KeyDownEvent {
    type Target = web_sys::KeyboardEvent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DomEvent for KeyDownEvent {
    fn event_type() -> crate::dom::Event {
        crate::dom::Event::KeyDown
    }

    fn from_dom(ev: web_sys::Event) -> Option<Self> {
        ev.dyn_into().ok().map(Self)
    }
}

make_str_enum! {
    Event {
        Cached = "cached",
        Error = "error",
        Abort = "abort",
        Load = "load",
        BeforeUnload = "beforeunload",
        Unload = "unload",
        Online = "online",
        Offline = "offline",
        Focus = "focus",
        Blur = "blur",
        Open = "open",
        Message = "message",
        Close = "close",
        PageHide = "pagehide",
        PageShow = "pageshow",
        PopState = "popstate",
        AnimationStart = "animationstart",
        AnimationEnd = "animationend",
        AnimationIteration = "animationiteration",
        TransitionStart = "transtionstart",
        TransitionEnd = "transitionend",
        TranstionRun = "transitionrun",
        Rest = "rest",
        Submit = "submit",
        BeforePrint = "beforeprint",
        AfterPrint = "afterprint",
        CompositionStart = "compositionstart",
        CompositionUpdate = "compositionupdate",
        CompositionEnd = "compositionend",
        FullScreenChange = "fullscreenchange",
        FullScreenError = "fullscreenerror",
        Resize = "resize",
        Scroll = "scroll",
        Cut = "cut",
        Copy = "copy",
        Paste = "paste",
        KeyDown = "keydown",
        KeyUp = "keyup",
        KeyPress = "keypress",
        AuxClick = "auxclick",
        Click = "click",
        ContextMenu = "contextmenu",
        DblClick = "dblclick",
        MouseDown = "mousedown",
        MouseEnter = "mouseenter",
        MouseLeave = "mouseleave",
        MouseMove = "mousemove",
        MouseOver = "mouseover",
        MouseOut = "mouseout",
        MouseUp = "mouseup",
        PointerLockChange = "pointerlockchange",
        PointerLockError = "pointerlockerror",
        Select = "select",
        Wheel = "wheel",
        PointerOver = "pointerover",
        PointerEnter = "pointerenter",
        PointerDown = "pointerdown",
        PointerMove = "pointermove",
        PointerUp = "pointerup",
        PointerCancel = "pointercancel",
        PointerOut = "pointerout",
        PointerLeave = "pointerleave",
        GotPointerCapture = "gotpointercapture",
        LostPointerCapture = "lostpointercapture",
        TouchStart = "touchstart",
        TouchEnd = "touchend",
        TouchCancel = "touchcancel",
        TouchMove = "touchmove",
        Drag = "drag",
        DragEnd = "dragend",
        DragEnter = "dragenter",
        DragStart = "dragstart",
        DragLeave = "dragleave",
        DragOver = "dragover",
        Drop = "drop",
        AudioProcess = "audioprocess",
        CanPlay = "canplay",
        CanPlayThrough = "canplaythrough",
        Complete = "complete",
        DurationChange = "durationchange",
        Emptied = "emptied",
        Ended = "ended",
        LoadedData = "loadeddata",
        LoadedMetaData = "loadedmetadata",
        Pause = "pause",
        Play = "play",
        Playing = "playing",
        RateChange = "ratechange",
        Seeked = "seeked",
        Seeking = "seeking",
        Stalled = "stalled",
        Suspend = "suspend",
        TimeUpdate = "timeupdate",
        VolumeChange = "volumechange",
        Waiting = "waiting",
        LoadEnd = "loadend",
        LoadStart = "loadstart",
        Timeout = "timeout",
        Change = "change",
        Storage = "storage",
        Checking = "checking",
        Downloading = "downloading",
        NoUpdate = "noupdate",
        Obselete = "obsolete",
        UpdateReady = "updateready",
        Broadcast = "broadcast",
        CheckBoxStateChange = "CheckBoxStateChange",
        HasChange = "haschange",
        Input = "input",
        RadioStateChange = "RadioStateChange",
        ReadyStateChange = "readystatechange",
        ValueChange = "ValueChange",
        Invalid = "invalid",
        Show = "show",
        SVGAbort = "SVGAbort",
        SVGError = "SVGError",
        SVGLoad = "SVGLoad",
        SVGResize = "SVGResize",
        SVGScroll = "SVGScroll",
        SVGUnload = "SVGUnload",
        Blocked = "blocked",
        Success = "success",
        UpgradeNeeded = "upgradeneeded",
        VersionChange = "versionchange",
        AfterScriptExecute = "afterscriptexecute",
        BeforeScriptExecute = "beforescriptexecute",
        DOMMenuItemActive = "DOMMenuItemActive",
        DOMMenuItemInactive = "DOMMenuItemInactive",
        PopupHidden = "popuphidden",
        PopupHiding = "popuphiding",
        PopupShowing = "popupshowing",
        PopupShown = "popupshown",
        VisibilityChange = "visibilitychange",
        ChargingChange = "chargingchange",
        ChargingTimeChange = "chargingtimechange",
        DischargingTimeChange = "dischargingtimechange",
        Connected = "connected",
        StateChange = "statechange",
        DeviceMotion = "devicemotion",
        DeviceOrientation = "deviceorientation",
        OrientationChange = "orientationchange",
        SmartCardInsert = "smartcard-insert",
        SmartCardRemove = "smartcard-remove",
        SelectionChange = "selectionchange",
    }
}
