//! An export instantiated in wasmi and called through ABI v1, as the loader calls it.

use wasmi::{Engine, Instance, Linker, Memory, Module, Store, WasmParams};

/// The status of a call that completed.
pub const DONE: u32 = 0;
/// `set_tag`'s index for the whole animation.
pub const WHOLE_ANIMATION: u32 = u32::MAX;
/// `tick`'s bit 0: the framebuffer changed.
pub const FRAME_CHANGED: u32 = 1;
/// `tick`'s bit 1: the range reached its end.
pub const RANGE_ENDED: u32 = 2;

/// The custom section that holds the payload.
const SECTION_NAME: &str = "life-pixel";
/// The ABI the player implements.
const ABI_VERSION: u32 = 1;
/// The bytes of one RGBA pixel.
const RGBA_BYTES: usize = 4;

/// An export's module, instantiated with its payload loaded.
pub struct WasmPlayer {
    store: Store<()>,
    instance: Instance,
    memory: Memory,
}

impl WasmPlayer {
    /// Compiles `export`, checks that it imports nothing, instantiates it, then takes the loader's
    /// steps: `abi_version`, `alloc`, a copy of its `life-pixel` section into memory, `load`.
    pub fn load(export: &[u8]) -> Self {
        let engine = Engine::default();
        let module = Module::new(&engine, export).unwrap();
        assert_eq!(module.imports().count(), 0, "an export imports nothing");
        let payload = payload(&module);
        let mut store = Store::new(&engine, ());
        let linker = Linker::new(&engine);
        let instance = linker.instantiate_and_start(&mut store, &module).unwrap();
        let memory = instance.get_memory(&store, "memory").unwrap();
        let mut player = Self {
            store,
            instance,
            memory,
        };
        assert_eq!(player.call("abi_version", ()), ABI_VERSION);
        let len = u32::try_from(payload.len()).unwrap();
        let pointer = player.call("alloc", (len,));
        assert_ne!(pointer, 0, "alloc({len}) returned 0");
        let address = usize::try_from(pointer).unwrap();
        player
            .memory
            .write(&mut player.store, address, &payload)
            .unwrap();
        assert_eq!(player.call("load", (pointer, len)), DONE, "load's status");
        player
    }

    /// Calls the export `name`: every ABI v1 export returns a `u32`.
    pub fn call<Params: WasmParams>(&mut self, name: &str, params: Params) -> u32 {
        let export = self
            .instance
            .get_typed_func::<Params, u32>(&self.store, name);
        let export = export.unwrap_or_else(|error| panic!("the export `{name}`: {error}"));
        export.call(&mut self.store, params).unwrap()
    }

    /// The framebuffer: `width × height` RGBA pixels.
    pub fn frame(&mut self) -> Vec<u8> {
        let pixels = self.call("width", ()) * self.call("height", ());
        let len = usize::try_from(pixels).unwrap() * RGBA_BYTES;
        let pointer = self.call("frame_ptr", ());
        self.read(pointer, len)
    }

    /// The title, read through `title_ptr` and `title_len`.
    pub fn title(&mut self) -> String {
        let (pointer, len) = (self.call("title_ptr", ()), self.call("title_len", ()));
        self.text(pointer, len)
    }

    /// The name of every tag, read through `tag_count`, `tag_name_ptr` and `tag_name_len`.
    pub fn tag_names(&mut self) -> Vec<String> {
        let tag_count = self.call("tag_count", ());
        (0..tag_count)
            .map(|index| {
                let pointer = self.call("tag_name_ptr", (index,));
                let len = self.call("tag_name_len", (index,));
                self.text(pointer, len)
            })
            .collect()
    }

    fn text(&self, pointer: u32, len: u32) -> String {
        String::from_utf8(self.read(pointer, usize::try_from(len).unwrap())).unwrap()
    }

    fn read(&self, pointer: u32, len: usize) -> Vec<u8> {
        let mut bytes = vec![0; len];
        let address = usize::try_from(pointer).unwrap();
        self.memory.read(&self.store, address, &mut bytes).unwrap();
        bytes
    }
}

/// The payload of `export`, compiled as the loader compiles it.
pub fn payload_of(export: &[u8]) -> Vec<u8> {
    payload(&Module::new(&Engine::default(), export).unwrap())
}

/// The payload: the content of the module's only `life-pixel` section, as the loader reads it.
fn payload(module: &Module) -> Vec<u8> {
    let sections: Vec<&[u8]> = module
        .custom_sections()
        .filter(|section| section.name() == SECTION_NAME)
        .map(|section| section.data())
        .collect();
    assert_eq!(
        sections.len(),
        1,
        "an export has one `{SECTION_NAME}` section"
    );
    sections[0].to_vec()
}
