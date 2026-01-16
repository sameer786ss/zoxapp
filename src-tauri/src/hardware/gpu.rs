//! GPU Detection for Windows
//!
//! Detects GPU hardware using multiple methods:
//! 1. NVIDIA: Check for nvidia-smi or registry
//! 2. AMD: Check for AMD registry entries
//! 3. Intel: Check for Intel graphics via registry
//! 4. Fallback to CPU if no GPU detected

use serde::{Deserialize, Serialize};
use std::process::Command;

/// GPU type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GpuType {
    Nvidia,
    Amd,
    Intel,
    Cpu,
}

impl GpuType {
    pub fn as_str(&self) -> &'static str {
        match self {
            GpuType::Nvidia => "nvidia",
            GpuType::Amd => "amd",
            GpuType::Intel => "intel",
            GpuType::Cpu => "cpu",
        }
    }
}

/// GPU detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub gpu_type: GpuType,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vram_mb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver_version: Option<String>,
}

impl GpuInfo {
    pub fn cpu_fallback() -> Self {
        GpuInfo {
            gpu_type: GpuType::Cpu,
            name: "CPU Only".to_string(),
            vram_mb: None,
            driver_version: None,
        }
    }
}

/// Main GPU detection function
/// Uses multiple methods to detect GPU hardware
pub fn detect_gpu() -> GpuInfo {
    println!("[GPU] Starting GPU detection...");

    // Try NVIDIA first (most common for ML)
    if let Some(info) = detect_nvidia() {
        println!("[GPU] Detected NVIDIA GPU: {}", info.name);
        return info;
    }

    // Try AMD
    if let Some(info) = detect_amd() {
        println!("[GPU] Detected AMD GPU: {}", info.name);
        return info;
    }

    // Try Intel iGPU
    if let Some(info) = detect_intel() {
        println!("[GPU] Detected Intel GPU: {}", info.name);
        return info;
    }

    // Fallback to WMI/PowerShell method
    if let Some(info) = detect_via_wmi() {
        println!("[GPU] Detected GPU via WMI: {}", info.name);
        return info;
    }

    // Fallback to CPU
    println!("[GPU] No GPU detected, falling back to CPU");
    GpuInfo::cpu_fallback()
}

/// Detect NVIDIA GPU using nvidia-smi
fn detect_nvidia() -> Option<GpuInfo> {
    // Try nvidia-smi first
    let output = Command::new("nvidia-smi")
        .args(["--query-gpu=name,memory.total,driver_version", "--format=csv,noheader,nounits"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next()?;
    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

    if parts.len() >= 3 {
        let name = parts[0].to_string();
        let vram_mb: u64 = parts[1].parse().ok()?;
        let driver_version = Some(parts[2].to_string());

        return Some(GpuInfo {
            gpu_type: GpuType::Nvidia,
            name,
            vram_mb: Some(vram_mb),
            driver_version,
        });
    }

    None
}

/// Detect AMD GPU via registry or PowerShell
fn detect_amd() -> Option<GpuInfo> {
    // Use PowerShell to query for AMD GPU
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            r#"Get-WmiObject Win32_VideoController | Where-Object { $_.Name -like '*AMD*' -or $_.Name -like '*Radeon*' } | Select-Object -First 1 Name, AdapterRAM, DriverVersion | ForEach-Object { "$($_.Name)|$($_.AdapterRAM)|$($_.DriverVersion)" }"#
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next()?.trim();
    
    if line.is_empty() {
        return None;
    }

    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() >= 3 {
        let name = parts[0].to_string();
        let vram_bytes: u64 = parts[1].parse().unwrap_or(0);
        let vram_mb = if vram_bytes > 0 { Some(vram_bytes / (1024 * 1024)) } else { None };
        let driver_version = if parts[2].is_empty() { None } else { Some(parts[2].to_string()) };

        return Some(GpuInfo {
            gpu_type: GpuType::Amd,
            name,
            vram_mb,
            driver_version,
        });
    }

    None
}

/// Detect Intel iGPU via PowerShell
fn detect_intel() -> Option<GpuInfo> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            r#"Get-WmiObject Win32_VideoController | Where-Object { $_.Name -like '*Intel*' } | Select-Object -First 1 Name, AdapterRAM, DriverVersion | ForEach-Object { "$($_.Name)|$($_.AdapterRAM)|$($_.DriverVersion)" }"#
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next()?.trim();
    
    if line.is_empty() {
        return None;
    }

    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() >= 3 {
        let name = parts[0].to_string();
        let vram_bytes: u64 = parts[1].parse().unwrap_or(0);
        let vram_mb = if vram_bytes > 0 { Some(vram_bytes / (1024 * 1024)) } else { None };
        let driver_version = if parts[2].is_empty() { None } else { Some(parts[2].to_string()) };

        return Some(GpuInfo {
            gpu_type: GpuType::Intel,
            name,
            vram_mb,
            driver_version,
        });
    }

    None
}

/// Generic WMI detection - returns first available GPU
fn detect_via_wmi() -> Option<GpuInfo> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            r#"Get-WmiObject Win32_VideoController | Select-Object -First 1 Name, AdapterRAM, DriverVersion | ForEach-Object { "$($_.Name)|$($_.AdapterRAM)|$($_.DriverVersion)" }"#
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next()?.trim();
    
    if line.is_empty() {
        return None;
    }

    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() >= 3 {
        let name = parts[0].to_string();
        let vram_bytes: u64 = parts[1].parse().unwrap_or(0);
        let vram_mb = if vram_bytes > 0 { Some(vram_bytes / (1024 * 1024)) } else { None };
        let driver_version = if parts[2].is_empty() { None } else { Some(parts[2].to_string()) };

        // Determine GPU type from name
        let gpu_type = if name.to_lowercase().contains("nvidia") || name.to_lowercase().contains("geforce") {
            GpuType::Nvidia
        } else if name.to_lowercase().contains("amd") || name.to_lowercase().contains("radeon") {
            GpuType::Amd
        } else if name.to_lowercase().contains("intel") {
            GpuType::Intel
        } else {
            GpuType::Cpu
        };

        return Some(GpuInfo {
            gpu_type,
            name,
            vram_mb,
            driver_version,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_gpu() {
        let info = detect_gpu();
        println!("Detected GPU: {:?}", info);
        // Just verify it doesn't panic
        assert!(!info.name.is_empty());
    }
}
