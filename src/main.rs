//! Sine wave generator with frequency configuration exposed through standard
//! input.

use std::io;
use std::str::FromStr;

mod effects;

fn main() {
    // 1. open a client
    let (client, _status) =
        jack::Client::new("dsw_experiment", jack::ClientOptions::NO_START_SERVER).unwrap();

    // 2. register port
    let in_a = client
        .register_port("in_l", jack::AudioIn::default())
        .unwrap();
    let in_b = client
        .register_port("in_r", jack::AudioIn::default())
        .unwrap();
    let mut out_a = client
        .register_port("out_l", jack::AudioOut::default())
        .unwrap();
    let mut out_b = client
        .register_port("out_r", jack::AudioOut::default())
        .unwrap();

    // 3. define process callback handler
    let sample_rate = client.sample_rate();
    let d = 100.0;
    let h = 200.0;

    let mut e = effects::Echo::new(d, h, sample_rate);

    let process = jack::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let in_a_p = in_a.as_slice(ps);
            let in_b_p = in_b.as_slice(ps);

            let out_a_p = out_a.as_mut_slice(ps);
            let out_b_p = out_b.as_mut_slice(ps);

            let ins = in_a_p.iter().zip(in_b_p);
            let outs = out_a_p.iter_mut().zip(out_b_p);

            for ((l_in, r_in), (l_out, r_out)) in ins.zip(outs)  {
                let in_sample = [*l_in, *r_in];

                let res = e.tick(in_sample);

                *l_out = res[0];
                *r_out = res[1];
            }

            jack::Control::Continue
        },
    );

    // 4. activate the client
    let active_client = client.activate_async((), process).unwrap();
    // processing starts here

    // 5. wait or do some processing while your handler is running in real time.
    println!("Enter an integer value to change the frequency of the sine wave.");
    while let Some(_) = read_freq() {
        
    }

    // 6. Optional deactivate. Not required since active_client will deactivate on
    // drop, though explicit deactivate may help you identify errors in
    // deactivate.
    active_client.deactivate().unwrap();
}

/// Attempt to read a frequency from standard in. Will block until there is
/// user input. `None` is returned if there was an error reading from standard
/// in, or the retrieved string wasn't a compatible u16 integer.
fn read_freq() -> Option<f64> {
    let mut user_input = String::new();
    match io::stdin().read_line(&mut user_input) {
        Ok(_) => u16::from_str(&user_input.trim()).ok().map(|n| n as f64),
        Err(_) => None,
    }
}