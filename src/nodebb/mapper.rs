use std::collections::HashMap;
use mongodb::coll::Collection;
use bson::Bson;
use std::rc::Rc;

pub struct NodeBB2PhpBBMapper {
    mongo_objects: Rc<Collection>,
    
    pub tid_map : HashMap<u32, Option<u32>>
}

impl NodeBB2PhpBBMapper  {
    pub fn new(coll: Rc<Collection>) -> NodeBB2PhpBBMapper {
        println!("Getting topic mappings, this will take a while...");
        let mut m = HashMap::new();
        let filter = doc! {
            "_key" => { "$regex" => r"^topic:" }
        };
        let cursor = coll.find(Some(filter.clone()), None).ok().expect("Failed to execute find topic");
        for item in cursor {
            match item {
                Ok(doc) => {
                    let bson_tid = doc.get("tid");
                    let bson_imported_tid = doc.get("_imported_tid");
                    match (bson_tid, bson_imported_tid) {
                        (Some(&Bson::I32(ref tid)),
                         Some(&Bson::I32(ref imported_tid))) => {
                            m.insert(tid.to_owned() as u32,
                                     Some(imported_tid.to_owned() as u32));
                        },
                        _ => {}
                    };
                }
                Err(_) => {
                    panic!("Failed to get next from server!");
                }
            }
        };
        println!("Got {} topic mappings", m.len());
        
        NodeBB2PhpBBMapper {
            tid_map: m,
            mongo_objects: coll
        }
    }

    fn set_tid(&mut self, tid: &u32, phpbb_tid: &Option<u32>) {
        self.tid_map.insert(tid.to_owned(), phpbb_tid.to_owned());
    }

    fn find_tid(&self, tid: &u32) -> Option<&Option<u32>> {
        self.tid_map.get(tid).clone()
    }
    
    pub fn get_tid(&mut self, tid: &u32) -> Option<u32> {
        let mut need_update = false;
        let rslt = match self.find_tid(tid) {
            Some(tid) => {
                tid.to_owned()
            },
            None => {
                let tid_owned = tid.to_owned();
                let filter = doc! {
                    "_key" => { "$regex" => r"^topic:" },
                    "tid" => tid_owned
                };
                println!("Search for tid {} ", tid);
                let mut cursor = self.mongo_objects.find(Some(filter.clone()), None).ok().expect("Failed to execute find topic");
                match cursor.next() {
                    Some(Ok(doc)) => {
                        let bson_imported_tid = doc.get("_imported_tid");
                        match bson_imported_tid {
                            Some(&Bson::I64(ref phpbb_tid)) => {
                                need_update = true;
                                Some(phpbb_tid.to_owned() as u32)
                            },
                            _ => None
                        }
                    },
                    Some(Err(_)) => {
                        panic!("Failed to get next from server!");
                    }
                    None => {
                        panic!("Failed to find tid {}", tid);
                    }
                }
                
            }
        };

        if need_update {
            self.set_tid(tid, &rslt);
        };
        
        rslt
    }
}

    
