use crate::Signal;

pub fn fft<S>(input: S) -> Vec<(f32, num::Complex<f32>)>
where
    S: Signal<Sample=num::Complex<f32>>,
{
    let rate = input.rate();
    let mut data: Vec<_> = input.iter().collect();
    let mut output = vec![num::Complex::new(0.0, 0.0); data.len()];
    let mut planner = rustfft::FFTplanner::new(false);
    let fft = planner.plan_fft(data.len());
    fft.process(&mut data, &mut output);

    let fstep = rate / (output.len() as f32);
    let start = -(output.len() as isize / 2);
    let norm = 1.0 / (output.len() as f32).sqrt();
    let mut collated = Vec::with_capacity(output.len());
    for i in 0..output.len() {
        let srci = start + i as isize;
        let srci_pos = if srci < 0 {
            srci + output.len() as isize
        } else {
            srci
        } as usize;
        collated.push((srci as f32 * fstep, output[srci_pos] * norm));
    }
    collated
}

pub fn rfft<S>(input: S) -> Vec<(f32, num::Complex<f32>)>
where
    S: Signal<Sample=f32>,
{
    let mut output = fft(input.map(|v| num::Complex::new(v, 0.0)));
    output.drain(0..output.len() / 2);
    output
}
