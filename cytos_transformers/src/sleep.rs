use std::{thread, time};

use cytos::{Prop, Result, Stepper};
use cytos_derive::CytosNode;

#[derive(CytosNode, Default)]
pub struct Sleep {
    #[input]
    millis: Prop<u64>,
}

impl Stepper for Sleep {
    fn step(&mut self) -> Result<()> {
        let ten_millis = time::Duration::from_millis(*self.millis);
        thread::sleep(ten_millis);

        Ok(())
    }
}
