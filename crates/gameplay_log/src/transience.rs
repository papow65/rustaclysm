use std::fmt;

pub trait LogMessageTransience: Clone + fmt::Debug + Send + Sync + 'static {}

#[derive(Clone, Debug)]
pub struct Intransient;

impl LogMessageTransience for Intransient {}

pub trait Transient: LogMessageTransience {}
