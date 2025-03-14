use std::{collections::VecDeque, sync::{Arc, Mutex}};

pub type AM<T> = Arc<Mutex<T>>;
pub type AMV<T> = Arc<Mutex<Vec<T>>>;
pub type AMQ<T> = Arc<Mutex<VecDeque<T>>>;
pub type AMO<T> = Arc<Mutex<Option<T>>>;
pub fn sync<T>(obj: T) -> AM<T> { Arc::new(Mutex::new(obj)) }
