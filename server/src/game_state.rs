pub mod game_state {
    use std::collections::HashMap;
    use std::time::Duration;
    use std::sync::{mpsc, Arc, RwLock};

    #[derive(Default, Debug, Clone, Copy)]
    pub struct Obj {
        pub x: u32,
        pub y: u32,
        pub dx: i32,
        pub dy: i32,
        pub id: u32,
    }

    #[derive(Debug, Clone)]
    pub enum Interaction {
        CreateEntity { x: u32, y: u32 },
        ClearEntities,
        MoveEntity { x: i32, y: i32, id: u32},
        CreateControlEntity { tx: mpsc::Sender<Obj> },
        DisconnectControlEntity { id: u32 }
    }

    pub fn game_loop(rx: mpsc::Receiver<Interaction>, objss: Arc<RwLock<Vec<Obj>>>) {
        let mut objs: Vec<Obj> = vec![];
        let mut objs: HashMap<u32, Obj> = HashMap::new();
        let mut next_id = 0;

        const target_frame: u32 = 20_000_000;
        // const target_frame: u32 = 2_000_000_000;

        let mut t = 0.0;
        let mut last = 0;
        let mut current_time = std::time::Instant::now();

        'game: loop {
            'recv_loop: loop {
                match rx.try_recv() {
                    Ok(v) => {
                        match v {
                            Interaction::CreateEntity { x, y } => {
                                objs.insert(next_id, (Obj {x: x, y: y, id: next_id, dx: 0, dy: 0}));
                                next_id += 1;
                                println!("{:?}", objs);
                            },
                            Interaction::ClearEntities => {
                                objs.clear();
                                next_id = 0;
                                // println!("Cleared Entities");
                            },
                            Interaction::MoveEntity { x, y, id } => {
                                // println!("move entity {:?}", v);
                                if objs.contains_key(&id) {
                                    objs.get_mut(&id).unwrap().dx = x;
                                    objs.get_mut(&id).unwrap().dy = y;
                                }
                            }
                            Interaction::CreateControlEntity { tx } => {
                                let ctrl_ent = Obj { x: 300, y: 300, id: next_id, dx: 0, dy: 0 };
                                let ctrl_ent_clone = ctrl_ent.clone();
                                objs.insert(next_id, ctrl_ent);
                                next_id += 1;
                                tx.send(ctrl_ent_clone);
                            },
                            Interaction::DisconnectControlEntity { id } => {
                                if objs.contains_key(&id) {
                                    objs.remove(&id);
                                }
                            }
                            _ => {}
                        }
                    },
                    Err(_) => break 'recv_loop
                }
            }
            last = current_time.elapsed().as_nanos() as u32;
            let last_secs = ((current_time.elapsed().as_nanos() as f64 / 1000000000.0)); // from ns to s
            // println!("{:?}", last_secs);
            
            current_time = std::time::Instant::now();
        
            for obj in objs.values_mut() {
                if obj.dx != 0 || obj.dy != 0 {
                    let mut _x = (obj.x as i32 + (obj.dx as f64 * last_secs * 100.0) as i32);
                    if _x < 0 {
                        _x = 0;
                    }
                    obj.x = _x as u32;
                    // obj.dx = 0;
                    let mut _y = (obj.y as i32 + (obj.dy as f64 * last_secs * 100.0) as i32);
                    if _y < 0 {
                        _y = 0;
                    }
                    obj.y = _y as u32;
                    // obj.dy = 0;
                } else {
                    obj.x += (1.0 * last_secs) as u32;
                }
            }

            // println!("{:?}", objs.values());

            {
                let mut objsss = objss.write().unwrap();
                objsss.clear();
                objsss.append(&mut(objs.clone().into_values().collect()));
            }

            t += last_secs;
            
            if target_frame > last {
                std::thread::sleep(Duration::new(0, target_frame - last));
            } else {
                println!("{:?}", last_secs);
            }
        }
    }
}