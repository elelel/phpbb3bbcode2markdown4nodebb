mod mapper;

use bson::Bson;
use mongodb::coll::Collection;
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use nodebb::mapper::NodeBB2PhpBBMapper;
use bbcode;
use std::rc::Rc;

pub struct NodeBB {
    mongo_objects: Rc<Collection>,
    forum_mapper: NodeBB2PhpBBMapper,
    bbcode_tags_query_re: String
}

impl NodeBB {
    pub fn new(mongo_hostname: &str,
               mongo_port: &u16,
               mongo_database: &str) -> NodeBB {
        let client = Client::connect(mongo_hostname, mongo_port.to_owned())
            .ok().expect("Failed to initialize mongodb client.");
        let coll = Rc::new(client.db(mongo_database).collection("objects"));

        let tags_query_re = {
            let mut re = "/".to_owned();
            let mut need_or = false;
            for tag in bbcode::KNOWN_TAGS.iter() {
                if need_or { re.push_str("|"); };
                need_or = true;
                re.push_str(r"\[");
                re.push_str(tag);
            }
            re.push_str("/");
            re
        };

        NodeBB {
            mongo_objects: coll.clone(),
            forum_mapper: NodeBB2PhpBBMapper::new(coll.clone()),
            bbcode_tags_query_re: tags_query_re
        }
    }

    pub fn convert_bbcode(&mut self, key_re: &str, target_field: &str) {
        let ref re = self.bbcode_tags_query_re;
        let filter = doc!
        { "_key" => { "$regex" => key_re },
            target_field => {
                "$regex" => re
            }
        };

        let cursor = self.mongo_objects.find(Some(filter.clone()), None).ok().expect("Failed to execute find.");
        for item in cursor {
            match item {
                Ok(doc) => {
                    let bson_key = doc.get("_key");
                    let bson_tid = doc.get("tid");
                    let bson_value = doc.get(target_field);

                    let phpbb_tid =
                        match bson_tid {
                            Some(&Bson::I32(ref tid)) => {
                                let tid_u32 = tid.to_owned() as u32;
                                self.forum_mapper.get_tid(&tid_u32)
                            }
                            _ =>{
                                Some(0.to_owned() as u32)
                            }
                        };

                    match (bson_key, bson_value) {
                        (Some(&Bson::String(ref key)),
                         Some(&Bson::String(ref value))) => {
                            //println!("SOURCE: {}", value);
                            let converted = bbcode::parse_text(phpbb_tid, value);
                            //println!("CONVERTED: {}\n", converted);
                            let update = doc! {
                                "$set" => {
                                    target_field => converted
                                }
                            };
                            let key_filter = doc! {
                                "_key" => key
                            };
                            let res = self.mongo_objects.update_one(key_filter, update, None);
                            if !res.is_ok() {
                                panic!("Update to mongo failed");
                            }
                            //                panic!("Record converted");
                        },
                        _ => panic!("Document for key regexp ".to_owned() + key_re +
                                    " does not contain valid string _key or " + target_field + " strings")
                    }
                    
                },
                Err(_) => {
                    panic!("Failed to get next from server!");
                },
            }
        }
        
    }
}
