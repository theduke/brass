pub type AnyBox = Box<dyn std::any::Any>;

pub fn any_box(value: impl std::any::Any) -> AnyBox {
    Box::new(value)
}
