mod calendar;
mod canvas;
use calendar::Calendar;

use sometime_core::{impl_registry_handlers, run_app};

impl_registry_handlers!(Calendar);

fn main() {
    run_app::<Calendar>();
}
