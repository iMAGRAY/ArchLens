#!/usr/bin/env cargo script

//! # Basic ArchLens Analysis Example
//! 
//! This example demonstrates how to use ArchLens programmatically
//! to analyze a project's architecture.

use std::process::Command;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏗️ ArchLens Basic Analysis Example");
    println!("=====================================");
    
    // Define the project path to analyze
    let project_path = std::env::current_dir()
        .unwrap_or_else(|_| {
            if cfg!(windows) {
                std::path::PathBuf::from("C:\\")
            } else {
                std::path::PathBuf::from("/tmp")
            }
        })
        .to_string_lossy()
        .to_string();
    
    // Check if ArchLens binary exists
    if !check_archlens_binary() {
        eprintln!("❌ ArchLens binary not found. Please build it first:");
        eprintln!("   cargo build --release");
        return Ok(());
    }
    
    println!("\n🔍 Running basic project analysis...");
    
    // Run basic analysis
    let binary_path = get_archlens_binary_path();
    let output = Command::new(&binary_path)
        .args(&["analyze", &project_path])
        .output()?;
    
    if output.status.success() {
        println!("✅ Analysis completed successfully!");
        println!("\n📊 Results:");
        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        eprintln!("❌ Analysis failed:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
    
    println!("\n🏗️ Getting project structure...");
    
    // Run structure analysis
    let structure_output = Command::new(&binary_path)
        .args(&["structure", &project_path, "--show-metrics"])
        .output()?;
    
    if structure_output.status.success() {
        println!("✅ Structure analysis completed!");
        println!("\n📁 Project Structure:");
        println!("{}", String::from_utf8_lossy(&structure_output.stdout));
    } else {
        eprintln!("❌ Structure analysis failed:");
        eprintln!("{}", String::from_utf8_lossy(&structure_output.stderr));
    }
    
    println!("\n🤖 Generating AI-ready export...");
    
    // Run AI compact export
    let ai_output = Command::new(&binary_path)
        .args(&["export", &project_path, "ai_compact"])
        .output()?;
    
    if ai_output.status.success() {
        println!("✅ AI export completed!");
        let ai_content = String::from_utf8_lossy(&ai_output.stdout);
        println!("\n🧠 AI Analysis Preview (first 500 chars):");
        println!("{}", &ai_content[..ai_content.len().min(500)]);
        if ai_content.len() > 500 {
            println!("... (truncated, {} total characters)", ai_content.len());
        }
    } else {
        eprintln!("❌ AI export failed:");
        eprintln!("{}", String::from_utf8_lossy(&ai_output.stderr));
    }
    
    println!("\n📈 Generating architecture diagram...");
    
    // Generate Mermaid diagram
    let diagram_output = Command::new(&binary_path)
        .args(&["diagram", &project_path, "mermaid", "--include-metrics"])
        .output()?;
    
    if diagram_output.status.success() {
        println!("✅ Diagram generated successfully!");
        println!("\n🎨 Mermaid Diagram:");
        println!("{}", String::from_utf8_lossy(&diagram_output.stdout));
    } else {
        eprintln!("❌ Diagram generation failed:");
        eprintln!("{}", String::from_utf8_lossy(&diagram_output.stderr));
    }
    
    println!("\n🎉 Analysis complete! Use the results above to understand your project's architecture.");
    
    Ok(())
}

fn check_archlens_binary() -> bool {
    let binary_path = get_archlens_binary_path();
    Path::new(&binary_path).exists()
}

fn get_archlens_binary_path() -> String {
    // Get absolute path to binary
    let current_dir = std::env::current_dir().unwrap_or_else(|_| {
        if cfg!(windows) {
            std::path::PathBuf::from("C:\\")
        } else {
            std::path::PathBuf::from("/tmp")
        }
    });
    
    let binary_name = if cfg!(windows) { "archlens.exe" } else { "archlens" };
    let binary_path = current_dir.join("target").join("release").join(binary_name);
    
    binary_path.to_string_lossy().to_string()
} 