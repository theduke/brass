#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tag {
    Address,
    Article,
    Aside,
    Footer,
    Header,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    Hgroup,
    Main,
    Nav,
    Section,
    BlockQuote,
    Dd,
    Dir,
    Div,
    Dl,
    Dt,
    FigCaption,
    Figure,
    Hr,
    Li,
    Ol,
    P,
    Pre,
    Ul,
    A,
    Abbr,
    B,
    Bdi,
    Bdo,
    Br,
    Cite,
    Code,
    Data,
    Dfn,
    Em,
    I,
    Kbd,
    Mark,
    Q,
    Rb,
    Rp,
    Rt,
    Rtc,
    Ruby,
    S,
    Samp,
    Small,
    Span,
    Strong,
    Sub,
    Sup,
    Time,
    Tt,
    U,
    Var,
    Wbr,
    Area,
    Audio,
    Img,
    Map,
    Track,
    Video,
    Applet,
    Embed,
    Iframe,
    NoEmbed,
    Object,
    Param,
    Picture,
    Source,
    Canvas,
    NoScript,
    Script,
    Del,
    Ins,
    Caption,
    Col,
    ColGroup,
    Table,
    Tbody,
    Td,
    Tfoot,
    Th,
    Thead,
    Tr,
    Button,
    DataList,
    FieldSet,
    Form,
    Input,
    Label,
    Legend,
    Meter,
    OptGroup,
    Option,
    Output,
    Progress,
    Select,
    TextArea,
    Details,
    Dialog,
    Menu,
    MenuItem,
    Summary,
    Content,
    Element,
    Shadow,
    Slot,
    Template,
    Animate,
    AnimateColor,
    AnimateMotion,
    AnimateTransform,
    Discard,
    Mpath,
    Set,
    Circle,
    Ellipse,
    Line,
    Polygon,
    Polyline,
    Rect,
    Mesh,
    Path,
    Defs,
    G,
    Marker,
    Mask,
    MissingGlyph,
    Pattern,
    Svg,
    Switch,
    Symbol,
    Unknown,
    Desc,
    Metadata,
    Title,
    FeBlend,
    FeColorMatrix,
    FeComponentTransfer,
    FeComposite,
    FeConvolveMatrix,
    FeDiffuseLighting,
    FeDisplacementMap,
    FeDropShadow,
    FeFlood,
    FeFuncA,
    FeFuncB,
    FeFuncG,
    FeFuncR,
    FeGaussianBlur,
    FeImage,
    FeMerge,
    FeMergeNode,
    FeMorphology,
    FeOffset,
    FeSpecularLighting,
    FeTile,
    FeTurbulence,
    FeDistantLight,
    FePointLight,
    FeSpotLight,
    Font,
    FontFace,
    FontFaceFormat,
    FontFaceName,
    FontFaceSrc,
    FontFaceUri,
    HKern,
    VKern,
    LinearGradient,
    MeshGradient,
    RadialGradient,
    Stop,
    Image,
    Use,
    Hatch,
    SolidColor,
    AltGlyph,
    AltGlyphDef,
    AltGlyphItem,
    Glyph,
    GlyphRef,
    TextPath,
    Text,
    TRef,
    TSpan,
    ClipPath,
    ColorProfile,
    Cursor,
    Filter,
    ForeignObject,
    HatchPath,
    MeshPatch,
    MeshRow,
    Style,
    View,
    Placeholder,
}

impl Tag {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Address => "address",
            Self::Article => "article",
            Self::Aside => "aside",
            Self::Footer => "footer",
            Self::Header => "header",
            Self::H1 => "h1",
            Self::H2 => "h2",
            Self::H3 => "h3",
            Self::H4 => "h4",
            Self::H5 => "h5",
            Self::H6 => "h6",
            Self::Hgroup => "hgroup",
            Self::Main => "main",
            Self::Nav => "nav",
            Self::Section => "section",
            Self::BlockQuote => "blockquote",
            Self::Dd => "dd",
            Self::Dir => "dir",
            Self::Div => "div",
            Self::Dl => "dl",
            Self::Dt => "dt",
            Self::FigCaption => "figcaption",
            Self::Figure => "figure",
            Self::Hr => "hr",
            Self::Li => "li",
            Self::Ol => "ol",
            Self::P => "p",
            Self::Pre => "pre",
            Self::Ul => "ul",
            Self::A => "a",
            Self::Abbr => "abbr",
            Self::B => "b",
            Self::Bdi => "bdi",
            Self::Bdo => "bdo",
            Self::Br => "br",
            Self::Cite => "cite",
            Self::Code => "code",
            Self::Data => "data",
            Self::Dfn => "dfn",
            Self::Em => "em",
            Self::I => "i",
            Self::Kbd => "kbd",
            Self::Mark => "mark",
            Self::Q => "q",
            Self::Rb => "rb",
            Self::Rp => "rp",
            Self::Rt => "rt",
            Self::Rtc => "rtc",
            Self::Ruby => "ruby",
            Self::S => "s",
            Self::Samp => "samp",
            Self::Small => "small",
            Self::Span => "span",
            Self::Strong => "strong",
            Self::Sub => "sub",
            Self::Sup => "sup",
            Self::Time => "time",
            Self::Tt => "tt",
            Self::U => "u",
            Self::Var => "var",
            Self::Wbr => "wbr",
            Self::Area => "area",
            Self::Audio => "audio",
            Self::Img => "img",
            Self::Map => "map",
            Self::Track => "track",
            Self::Video => "video",
            Self::Applet => "applet",
            Self::Embed => "embed",
            Self::Iframe => "iframe",
            Self::NoEmbed => "noembed",
            Self::Object => "object",
            Self::Param => "param",
            Self::Picture => "picture",
            Self::Source => "source",
            Self::Canvas => "canvas",
            Self::NoScript => "noscript",
            Self::Script => "script",
            Self::Del => "del",
            Self::Ins => "ins",
            Self::Caption => "caption",
            Self::Col => "col",
            Self::ColGroup => "colgroup",
            Self::Table => "table",
            Self::Tbody => "tbody",
            Self::Td => "td",
            Self::Tfoot => "tfoot",
            Self::Th => "th",
            Self::Thead => "thead",
            Self::Tr => "tr",
            Self::Button => "button",
            Self::DataList => "datalist",
            Self::FieldSet => "fieldset",
            Self::Form => "form",
            Self::Input => "input",
            Self::Label => "label",
            Self::Legend => "legend",
            Self::Meter => "meter",
            Self::OptGroup => "optgroup",
            Self::Option => "option",
            Self::Output => "output",
            Self::Progress => "progress",
            Self::Select => "select",
            Self::TextArea => "textarea",
            Self::Details => "details",
            Self::Dialog => "dialog",
            Self::Menu => "menu",
            Self::MenuItem => "menuitem",
            Self::Summary => "summary",
            Self::Content => "content",
            Self::Element => "element",
            Self::Shadow => "shadow",
            Self::Slot => "slot",
            Self::Template => "template",
            Self::Animate => "animate",
            Self::AnimateColor => "animatecolor",
            Self::AnimateMotion => "animatemotion",
            Self::AnimateTransform => "animatetransform",
            Self::Discard => "discard",
            Self::Mpath => "mpath",
            Self::Set => "set",
            Self::Circle => "circle",
            Self::Ellipse => "ellipse",
            Self::Line => "line",
            Self::Polygon => "polygon",
            Self::Polyline => "polyline",
            Self::Rect => "rect",
            Self::Mesh => "mesh",
            Self::Path => "path",
            Self::Defs => "defs",
            Self::G => "g",
            Self::Marker => "marker",
            Self::Mask => "mask",
            Self::MissingGlyph => "missingglyph",
            Self::Pattern => "pattern",
            Self::Svg => "svg",
            Self::Switch => "switch",
            Self::Symbol => "symbol",
            Self::Unknown => "unknown",
            Self::Desc => "desc",
            Self::Metadata => "metadata",
            Self::Title => "title",
            Self::FeBlend => "feblend",
            Self::FeColorMatrix => "fecolormatrix",
            Self::FeComponentTransfer => "fecomponenttransfer",
            Self::FeComposite => "fecomposite",
            Self::FeConvolveMatrix => "feconvolvematrix",
            Self::FeDiffuseLighting => "fediffuselighting",
            Self::FeDisplacementMap => "fedisplacementmap",
            Self::FeDropShadow => "fedropshadow",
            Self::FeFlood => "feflood",
            Self::FeFuncA => "fefunca",
            Self::FeFuncB => "fefuncb",
            Self::FeFuncG => "fefuncg",
            Self::FeFuncR => "fefuncr",
            Self::FeGaussianBlur => "fegaussianblur",
            Self::FeImage => "feimage",
            Self::FeMerge => "femerge",
            Self::FeMergeNode => "femergenode",
            Self::FeMorphology => "femorphology",
            Self::FeOffset => "feoffset",
            Self::FeSpecularLighting => "fespecularlighting",
            Self::FeTile => "fetile",
            Self::FeTurbulence => "feturbulence",
            Self::FeDistantLight => "fedistantlight",
            Self::FePointLight => "fepointlight",
            Self::FeSpotLight => "fespotlight",
            Self::Font => "font",
            Self::FontFace => "fontface",
            Self::FontFaceFormat => "fontfaceformat",
            Self::FontFaceName => "fontfacename",
            Self::FontFaceSrc => "fontfacesrc",
            Self::FontFaceUri => "fontfaceuri",
            Self::HKern => "hkern",
            Self::VKern => "vkern",
            Self::LinearGradient => "lineargradient",
            Self::MeshGradient => "meshgradient",
            Self::RadialGradient => "radialgradient",
            Self::Stop => "stop",
            Self::Image => "image",
            Self::Use => "use",
            Self::Hatch => "hatch",
            Self::SolidColor => "solidcolor",
            Self::AltGlyph => "altglyph",
            Self::AltGlyphDef => "altglyphdef",
            Self::AltGlyphItem => "altglyphitem",
            Self::Glyph => "glyph",
            Self::GlyphRef => "glyphref",
            Self::TextPath => "textpath",
            Self::Text => "text",
            Self::TRef => "tref",
            Self::TSpan => "tspan",
            Self::ClipPath => "clippath",
            Self::ColorProfile => "colorprofile",
            Self::Cursor => "cursor",
            Self::Filter => "filter",
            Self::ForeignObject => "foreignobject",
            Self::HatchPath => "hatchpath",
            Self::MeshPatch => "meshpatch",
            Self::MeshRow => "meshrow",
            Self::Style => "style",
            Self::View => "view",
            Self::Placeholder => "placeholder",
        }
    }
}
