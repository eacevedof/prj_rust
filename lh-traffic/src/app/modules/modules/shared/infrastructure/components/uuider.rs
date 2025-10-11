use uuid::Uuid;

pub struct Uuider;

impl Uuider {
    pub fn generate() -> String {
        Uuid::new_v4().to_string()
    }
}
