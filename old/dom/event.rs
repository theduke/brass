
crate::make_str_enum! {
    Event {
        Cached = 0 = "cached",
        Error = 1 = "error",
        Abort = 2 = "abort",
        Load = 3 = "load",
        BeforeUnload = 4 = "beforeunload",
        Unload = 5 = "unload",
        Online = 6 = "online",
        Offline = 7 = "offline",
        Focus = 8 = "focus",
        Blur = 9 = "blur",
        Open = 10 = "open",
        Message = 11 = "message",
        Close = 12 = "close",
        PageHide = 13 = "pagehide",
        PageShow = 14 = "pageshow",
        PopState = 15 = "popstate",
        AnimationStart = 16 = "animationstart",
        AnimationEnd = 17 = "animationend",
        AnimationIteration = 18 = "animationiteration",
        TransitionStart = 19 = "transtionstart",
        TransitionEnd = 20 = "transitionend",
        TranstionRun = 21 = "transitionrun",
        Rest = 22 = "rest",
        Submit = 23 = "submit",
        BeforePrint = 24 = "beforeprint",
        AfterPrint = 25 = "afterprint",
        CompositionStart = 26 = "compositionstart",
        CompositionUpdate = 27 = "compositionupdate",
        CompositionEnd = 28 = "compositionend",
        FullScreenChange = 29 = "fullscreenchange",
        FullScreenError = 30 = "fullscreenerror",
        Resize = 31 = "resize",
        Scroll = 32 = "scroll",
        Cut = 33 = "cut",
        Copy = 34 = "copy",
        Paste = 35 = "paste",
        KeyDown = 36 = "keydown",
        KeyUp = 37 = "keyup",
        KeyPress = 38 = "keypress",
        AuxClick = 39 = "auxclick",
        Click = 40 = "click",
        ContextMenu = 41 = "contextmenu",
        DblClick = 42 = "dblclick",
        MouseDown = 43 = "mousedown",
        MouseEnter = 44 = "mouseenter",
        MouseLeave = 45 = "mouseleave",
        MouseMove = 46 = "mousemove",
        MouseOver = 47 = "mouseover",
        MouseOut = 48 = "mouseout",
        MouseUp = 49 = "mouseup",
        PointerLockChange = 50 = "pointerlockchange",
        PointerLockError = 51 = "pointerlockerror",
        Select = 52 = "select",
        Wheel = 53 = "wheel",
        PointerOver = 54 = "pointerover",
        PointerEnter = 55 = "pointerenter",
        PointerDown = 56 = "pointerdown",
        PointerMove = 57 = "pointermove",
        PointerUp = 58 = "pointerup",
        PointerCancel = 59 = "pointercancel",
        PointerOut = 60 = "pointerout",
        PointerLeave = 61 = "pointerleave",
        GotPointerCapture = 62 = "gotpointercapture",
        LostPointerCapture = 63 = "lostpointercapture",
        TouchStart = 64 = "touchstart",
        TouchEnd = 65 = "touchend",
        TouchCancel = 66 = "touchcancel",
        TouchMove = 67 = "touchmove",
        Drag = 68 = "drag",
        DragEnd = 69 = "dragend",
        DragEnter = 70 = "dragenter",
        DragStart = 71 = "dragstart",
        DragLeave = 72 = "dragleave",
        DragOver = 73 = "dragover",
        Drop = 74 = "drop",
        AudioProcess = 75 = "audioprocess",
        CanPlay = 76 = "canplay",
        CanPlayThrough = 77 = "canplaythrough",
        Complete = 78 = "complete",
        DurationChange = 79 = "durationchange",
        Emptied = 80 = "emptied",
        Ended = 81 = "ended",
        LoadedData = 82 = "loadeddata",
        LoadedMetaData = 83 = "loadedmetadata",
        Pause = 84 = "pause",
        Play = 85 = "play",
        Playing = 86 = "playing",
        RateChange = 87 = "ratechange",
        Seeked = 88 = "seeked",
        Seeking = 89 = "seeking",
        Stalled = 90 = "stalled",
        Suspend = 91 = "suspend",
        TimeUpdate = 92 = "timeupdate",
        VolumeChange = 93 = "volumechange",
        Waiting = 94 = "waiting",
        LoadEnd = 95 = "loadend",
        LoadStart = 96 = "loadstart",
        Timeout = 97 = "timeout",
        Change = 98 = "change",
        Storage = 99 = "storage",
        Checking = 100 = "checking",
        Downloading = 101 = "downloading",
        NoUpdate = 102 = "noupdate",
        Obselete = 103 = "obsolete",
        UpdateReady = 104 = "updateready",
        Broadcast = 105 = "broadcast",
        CheckBoxStateChange = 106 = "CheckBoxStateChange",
        HasChange = 107 = "haschange",
        Input = 108 = "input",
        RadioStateChange = 109 = "RadioStateChange",
        ReadyStateChange = 110 = "readystatechange",
        ValueChange = 111 = "ValueChange",
        Invalid = 112 = "invalid",
        Show = 113 = "show",
        SVGAbort = 114 = "SVGAbort",
        SVGError = 115 = "SVGError",
        SVGLoad = 116 = "SVGLoad",
        SVGResize = 117 = "SVGResize",
        SVGScroll = 118 = "SVGScroll",
        SVGUnload = 119 = "SVGUnload",
        Blocked = 120 = "blocked",
        Success = 121 = "success",
        UpgradeNeeded = 122 = "upgradeneeded",
        VersionChange = 123 = "versionchange",
        AfterScriptExecute = 124 = "afterscriptexecute",
        BeforeScriptExecute = 125 = "beforescriptexecute",
        DOMMenuItemActive = 126 = "DOMMenuItemActive",
        DOMMenuItemInactive = 127 = "DOMMenuItemInactive",
        PopupHidden = 128 = "popuphidden",
        PopupHiding = 129 = "popuphiding",
        PopupShowing = 130 = "popupshowing",
        PopupShown = 131 = "popupshown",
        VisibilityChange = 132 = "visibilitychange",
        ChargingChange = 133 = "chargingchange",
        ChargingTimeChange = 134 = "chargingtimechange",
        DischargingTimeChange = 135 = "dischargingtimechange",
        Connected = 136 = "connected",
        StateChange = 137 = "statechange",
        DeviceMotion = 138 = "devicemotion",
        DeviceOrientation = 139 = "deviceorientation",
        OrientationChange = 140 = "orientationchange",
        SmartCardInsert = 141 = "smartcard-insert",
        SmartCardRemove = 142 = "smartcard-remove",
        ;
        SelectionChange = 143 = "selectionchange"
    }
}