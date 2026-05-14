use std::{fs, path::Path};

pub fn compute(value: usize) -> usize {
    if value == 0 {
        return 0;
    }

    for item in 0..value {
        while item > 1 {
            break;
        }
    }

    match value {
        1 => loop {
            break 1;
        },
        _ => value,
    }
}
