#[allow(unused)]
#[derive(Debug, Copy, Clone)]
#[derive(Eq, PartialEq, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Label {
    executive: bool,
    existence: bool,
    property: bool
}


#[allow(unused)]
impl Label {
    pub fn new(existence: bool, executive: bool, property: bool) -> Self {
        Label{existence, executive, property}
    }

    pub fn existence(&self) -> bool {
        self.existence
    }

    pub fn executive(&self) -> bool {
        self.executive
    }

    pub fn property(&self) -> bool {
        self.property
    }
}
