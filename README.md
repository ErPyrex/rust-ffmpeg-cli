# 🚀 rust-ffmpeg-cli

**rust-ffmpeg-cli** es una potente herramienta de línea de comandos (CLI) escrita en Rust que actúa como un optimizador inteligente de archivos multimedia (Video, Audio e Imágenes) utilizando FFmpeg bajo el capó. 

Está diseñada para facilitar la optimización de archivos para plataformas específicas como Discord, WhatsApp, YouTube y Telegram, ofreciendo tanto un modo interactivo guiado como comandos directos para usuarios avanzados.

## ✨ Características

- 🎥 **Video**: Conversión inteligente a H.264 (MP4) con perfiles optimizados para diferentes plataformas.
- 🖼️ **Imágenes**: Optimización y conversión a WebP o JPEG con control de calidad.
- 🎵 **Audio**: Ajuste automático de bitrate según el modo de calidad.
- 🤖 **Modo Interactivo**: Interfaz amigable que te guía paso a paso si no proporcionas argumentos.
- ⚡ **Alto Rendimiento**: Desarrollado en Rust para una ejecución rápida y eficiente.
- 📊 **Resumen de Ahorro**: Muestra el tamaño original vs. el optimizado y el porcentaje de espacio ahorrado.

## 🛠️ Requisitos

Para que esta herramienta funcione, debes tener instalado **FFmpeg** en tu sistema y disponible en tu PATH.

## 🚀 Instalación

Si tienes Rust instalado, puedes compilarlo directamente:

```bash
git clone https://github.com/tu-usuario/rust-ffmpeg-cli.git
cd rust-ffmpeg-cli
cargo build --release
```

El binario resultante se encontrará en `target/release/rust-ffmpeg-cli`.

## 📖 Uso

### Modo Interactivo
Simplemente ejecuta el programa sin argumentos para iniciar el asistente:
```bash
./rust-ffmpeg-cli
```

### Comandos Directos
También puedes usarlo directamente especificando las opciones:

```bash
# Optimizar video para DISCORD (Límite de 25MB)
rust-ffmpeg-cli -i video.mp4 -o discord.mp4 --platform discord --mode light

# Preparar video para WHATSAPP
rust-ffmpeg-cli -i vacaciones.mov -o whatsapp.mp4 --platform whatsapp --mode balanced

# Renderizar para YOUTUBE en Alta Calidad
rust-ffmpeg-cli -i master.mkv -o youtube.mp4 --platform youtube --mode high
```

### Argumentos disponibles:
- `-i, --input <INPUT>`: Archivo de entrada.
- `-o, --output <OUTPUT>`: Archivo de salida (opcional).
- `-p, --platform <PLATFORM>`: Plataforma de destino (`generic`, `discord`, `whatsapp`, `youtube`, `telegram`).
- `-m, --mode <MODE>`: Modo de calidad (`light`, `balanced`, `high`).
- `--width <WIDTH>`: Escalar el ancho manualmente.
- `--height <HEIGHT>`: Escalar el alto manualmente.
- `--quality <QUALITY>`: Calidad manual (CRF para video, Q para imágenes).
- `-v, --verbose`: Muestra la salida técnica de FFmpeg.

## 📜 Licencia

Este proyecto está bajo la licencia MIT.
