/// A package on either side of the boundary.
#[derive(Clone, Debug)]
pub struct Member {
    /// The package name, `life-pixel-format` for example.
    pub name: String,
    /// The licence its manifest declares, if any.
    pub license: Option<String>,
    /// The packages it depends on to build or run; development dependencies are left out,
    /// since they never ship.
    pub dependencies: Vec<String>,
}
