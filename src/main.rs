use crossbeam_channel::bounded;
use text_io::scan;

mod delay_line;
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
    let d = 10.0;
    let h = 20.0;

    let (tx, rx) = bounded::<effects::EchoParameters>(1_000_000);

    //let mut e = effects::Echo::new(d, h, sample_rate, 64000);
    let mut e = effects::Flange::new(2.0, 0.005, 0.8, sample_rate);

    let process = jack::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let in_a_p = in_a.as_slice(ps);
            let in_b_p = in_b.as_slice(ps);

            let out_a_p = out_a.as_mut_slice(ps);
            let out_b_p = out_b.as_mut_slice(ps);

            let ins = in_a_p.iter().zip(in_b_p);
            let outs = out_a_p.iter_mut().zip(out_b_p);

            while let Ok(f) = rx.try_recv() {
                //e.set_params(f);
            }

            for ((l_in, r_in), (l_out, r_out)) in ins.zip(outs) {
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
    println!("Enter d and h values");
    loop {
        let d: f32;
        let h: f32;
        scan!("{} {}\n", d, h);

        if d < 0.1 && h < 0.1 {
            break;
        }

        println!("read d: {} and h: {}", d, h);

        let params = effects::EchoParameters::from_distances(d, h, sample_rate);

        println!(
            "That gives params: g: {}, m: {}",
            params.attenuation, params.length
        );

        tx.send(effects::EchoParameters::from_distances(d, h, sample_rate))
            .unwrap();
    }

    // 6. Optional deactivate. Not required since active_client will deactivate on
    // drop, though explicit deactivate may help you identify errors in
    // deactivate.
    active_client.deactivate().unwrap();
}
