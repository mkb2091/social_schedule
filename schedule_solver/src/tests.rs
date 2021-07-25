use crate::Schedule;
const GROUPS: &[usize] = &[4; 6];

const SCHEDULER: Schedule = Schedule::new(GROUPS, GROUPS.len());

#[quickcheck_macros::quickcheck]
fn check_step(mut buf_1: Vec<usize>, mut buf_2: Vec<usize>) {
        SCHEDULER.step(&mut buf_1, &mut buf_2);
}
