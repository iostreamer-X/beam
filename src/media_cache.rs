pub struct MediaCache(Option<String>);

impl MediaCache {
    pub fn update(&mut self, id: &String) -> bool {
        let did_media_change = self.0.as_ref().map_or(true, |c| c.cmp(id).is_ne());
        self.0.replace(id.clone());
        return did_media_change;
    }

    pub fn clear(&mut self) -> &mut MediaCache {
        self.0 = None;
        return self;
    }

    pub fn init() -> Self {
        return Self(None);
    }
}
