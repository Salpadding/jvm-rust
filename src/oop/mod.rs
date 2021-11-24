use crate::heap::class::Object;
use rp::Rp;

pub type jobject = Rp<Object>;
pub type jarray = jobject;
pub type jstring = jobject;
