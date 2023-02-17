#[macro_export]
macro_rules! cast {
    ($self: ident, $type: ident) => {
        $self.as_any().downcast_ref::<$type>().unwrap()
    };
}


