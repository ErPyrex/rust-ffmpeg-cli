use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use console::{style, Term};
use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(Parser, Debug)]
#[command(
    name = "rust-ffmpeg-cli",
    author = "ErPyrex",
    version = "1.3",
    about = "🚀 Optimizador inteligente de medios (Video, Audio e Imágenes) usando FFmpeg",
    after_help = " EJEMPLOS DE USO:
    
    1. Optimizar video para DISCORD:
       $ rust-ffmpeg-cli -i video.mp4 -o discord.mp4 --platform discord --mode light

    2. Preparar video para WHATSAPP:
       $ rust-ffmpeg-cli -i vacaciones.mov -o whatsapp.mp4 --platform whatsapp --mode balanced

    3. Renderizar para YOUTUBE en Alta Calidad:
       $ rust-ffmpeg-cli -i master.mkv -o youtube.mp4 --platform youtube --mode high
    "
)]
struct Args {
    /// Archivo de entrada
    #[arg(short, long)]
    input: Option<String>,

    /// Archivo de salida
    #[arg(short, long)]
    output: Option<String>,

    /// Plataforma de destino
    #[arg(short, long, value_enum)]
    platform: Option<Platform>,

    /// Modo de optimización
    #[arg(short, long, value_enum)]
    mode: Option<QualityMode>,

    /// Escalar el ancho manualmente
    #[arg(long)]
    width: Option<i32>,

    /// Escalar el alto manualmente
    #[arg(long)]
    height: Option<i32>,

    /// Calidad manual (CRF)
    #[arg(long)]
    quality: Option<u32>,

    /// Muestra la salida técnica de FFmpeg
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug, Display, EnumIter)]
#[strum(serialize_all = "title_case")]
enum Platform {
    Generic,
    Discord,
    Whatsapp,
    Youtube,
    Telegram,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug, Display, EnumIter)]
#[strum(serialize_all = "title_case")]
enum QualityMode {
    Light,
    Balanced,
    High,
}

enum MediaType { Video, Audio, Image }

fn main() -> Result<()> {
    let args = Args::parse();

    // Determinar si activar modo interactivo o CLI directo
    let (input, output, platform, mode) = if args.input.is_none() {
        run_interactive_mode()?
    } else {
        (
            args.input.clone().context("No se proporcionó archivo de entrada")?,
            args.output.clone().unwrap_or_else(|| "output.mp4".to_string()),
            args.platform.unwrap_or(Platform::Generic),
            args.mode.unwrap_or(QualityMode::Balanced),
        )
    };

    let input_path = Path::new(&input);
    if !input_path.exists() {
        anyhow::bail!("❌ No existe el archivo de entrada: {}", input);
    }

    let media_type = detect_media_type(&input);
    let mut command = Command::new("ffmpeg");
    command.arg("-hide_banner").arg("-y");
    if !args.verbose { command.arg("-loglevel").arg("error"); }
    command.arg("-i").arg(&input);

    match media_type {
        MediaType::Video => {
            command.arg("-c:v").arg("libx264").arg("-pix_fmt").arg("yuv420p");
            command.arg("-c:a").arg("aac");

            let max_width = match platform {
                Platform::Whatsapp => Some(1080),
                Platform::Discord if mode == QualityMode::Light => Some(720),
                Platform::Telegram => Some(1080),
                _ => None,
            };

            let final_w = args.width.or(max_width);
            if let Some(w) = final_w {
                command.arg("-vf").arg(format!("scale={}:-2", w));
            } else if let Some(h) = args.height {
                command.arg("-vf").arg(format!("scale=-2:{}", h));
            }

            let crf = match (platform, mode) {
                (_, QualityMode::Light) => 28,
                (Platform::Youtube, QualityMode::High) => 18,
                (_, QualityMode::High) => 20,
                _ => 24,
            };
            command.arg("-crf").arg(args.quality.unwrap_or(crf).to_string());

            match platform {
                Platform::Discord if mode == QualityMode::Light => {
                    command.arg("-fs").arg("24M");
                }
                Platform::Whatsapp => {
                    command.arg("-profile:v").arg("baseline").arg("-level").arg("3.0");
                }
                Platform::Youtube => {
                    command.arg("-preset").arg("slow").arg("-bf").arg("2");
                }
                _ => { command.arg("-preset").arg("medium"); }
            }

            if mode == QualityMode::Light {
                command.arg("-r").arg("24");
            }
        }

        MediaType::Image => {
            let out_ext = get_extension(&output);
            match out_ext.as_str() {
                "webp" => {
                    command.arg("-c:v").arg("libwebp");
                    let q = match mode {
                        QualityMode::Light => 50,
                        QualityMode::Balanced => 75,
                        QualityMode::High => 95,
                    };
                    command.arg("-q:v").arg(args.quality.unwrap_or(q).to_string());
                }
                "jpg" | "jpeg" => {
                    command.arg("-c:v").arg("mjpeg");
                    let q = match mode {
                        QualityMode::Light => 6,
                        QualityMode::Balanced => 3,
                        QualityMode::High => 1,
                    };
                    command.arg("-q:v").arg(args.quality.unwrap_or(q).to_string());
                }
                _ => {}
            }
            
            if let Some(w) = args.width.or(match platform { Platform::Whatsapp => Some(1200), _ => None }) {
                command.arg("-vf").arg(format!("scale={}:-1", w));
            }
        }

        MediaType::Audio => {
            let bitrate = match mode {
                QualityMode::Light => "64k",
                QualityMode::Balanced => "128k",
                QualityMode::High => "320k",
            };
            command.arg("-b:a").arg(bitrate);
        }
    }

    command.arg(&output);

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} [{elapsed_precise}] {msg}")?);
    pb.set_message(format!("Procesando {}...", style(&input).yellow()));
    pb.enable_steady_tick(Duration::from_millis(100));

    let mut child = if args.verbose {
        command.stdout(Stdio::inherit()).stderr(Stdio::inherit()).spawn()?
    } else {
        command.stdout(Stdio::null()).stderr(Stdio::null()).spawn()?
    };

    let status = child.wait().context("Error al ejecutar FFmpeg")?;
    pb.finish_and_clear();

    if status.success() {
        println!("{}", style("✨ ¡Optimización terminada con éxito!").green().bold());
        println!("📂 Guardado en: {}", style(&output).cyan());
        if let (Ok(m1), Ok(m2)) = (std::fs::metadata(&input), std::fs::metadata(&output)) {
            let s1 = m1.len() as f64 / 1_048_576.0;
            let s2 = m2.len() as f64 / 1_048_576.0;
            println!("📊 Resumen: {:.2}MB -> {:.2}MB ({:.1}% ahorro)", s1, s2, (1.0 - s2/s1)*100.0);
        }
    } else {
        println!("{}", style("❌ Falló el procesamiento de FFmpeg.").red());
    }

    Ok(())
}

fn run_interactive_mode() -> Result<(String, String, Platform, QualityMode)> {
    let term = Term::stdout();
    term.clear_screen()?;

    println!("{}", style("╔════════════════════════════════════════════════╗").cyan());
    println!("{}", style("║          🚀 RUST MULTIMEDIA OPTIMIZER          ║").cyan().bold());
    println!("{}", style("╚════════════════════════════════════════════════╝").cyan());
    println!();

    // 1. Archivo de entrada
    let input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("📝 Arrastra el archivo aquí o escribe su ruta")
        .validate_with(|input: &String| -> Result<(), &str> {
            let path_str = input.trim().trim_matches('\'').trim_matches('"');
            if Path::new(path_str).exists() {
                Ok(())
            } else {
                Err("El archivo no existe. Por favor, verifica la ruta.")
            }
        })
        .interact_text()?;

    let clean_input = input.trim().trim_matches('\'').trim_matches('"').to_string();

    // Sugerir salida
    let input_path = Path::new(&clean_input);
    let stem = input_path.file_stem().map(|s| s.to_string_lossy()).unwrap_or_else(|| "output".into());
    let ext = input_path.extension().map(|s| s.to_string_lossy()).unwrap_or_else(|| "mp4".into());
    let default_output = format!("{}_opt.{}", stem, ext);

    // 2. Archivo de salida
    let output: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("💾 Nombre del archivo de salida")
        .default(default_output)
        .interact_text()?;

    // 3. Plataforma
    let platforms: Vec<_> = Platform::iter().collect();
    let platform_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("📱 Selecciona la plataforma de destino")
        .items(&platforms)
        .default(0)
        .interact()?;
    let platform = platforms[platform_idx];

    // 4. Modo de Calidad
    let modes: Vec<_> = QualityMode::iter().collect();
    let mode_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("🎯 ¿Qué objetivo tienes?")
        .items(&modes)
        .default(1)
        .interact()?;
    let mode = modes[mode_idx];

    println!();
    Ok((clean_input, output, platform, mode))
}

fn detect_media_type(path: &str) -> MediaType {
    let ext = get_extension(path);
    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp" => MediaType::Image,
        "mp3" | "wav" | "ogg" | "m4a" | "flac" | "aac" => MediaType::Audio,
        _ => MediaType::Video,
    }
}

fn get_extension(path: &str) -> String {
    Path::new(path).extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase()
}
