fn main() {
    let workspace = std::path::PathBuf::from(r"C:\Users\Sameer\AppData\Roaming\zox\workspace");
    let file = workspace.join("BankAccount.java");
    
    // Simulate what WriteFileTool does
    println!("Workspace: {}", workspace.display());
    println!("File path: {}", file.display());
    
    // Check if simple string path resolves correctly
    let rel_path = "BankAccount.java";
    let full_path = workspace.join(rel_path);
    println!("Resolved: {}", full_path.display());
    
    // Test if file exists (it should, based on logs)
    if full_path.exists() {
        println!("File exists!");
    } else {
        println!("File NOT found at expected path!");
    }

    // Simulate search
    let walker = ignore::WalkBuilder::new(&workspace)
        .build();
        
    println!("Walking workspace...");
    for result in walker {
        match result {
            Ok(entry) => println!("Entry: {}", entry.path().display()),
            Err(err) => println!("ERROR: {}", err),
        }
    }
}
