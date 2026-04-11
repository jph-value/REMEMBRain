fn main() {
    // Track clone/build events
    rememnemosyne_clone_tracker::track_clone(
        rememnemosyne_clone_tracker::TrackerConfig::default()
    ).ok();
    
    println!("cargo:rerun-if-changed=build.rs");
}
