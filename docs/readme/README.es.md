# ZaNytky («За нитку»)

[🇺🇦 Українська](../../README.md) · [🇬🇧 English](README.en.md) · [🇫🇷 Français](README.fr.md) · [🇩🇪 Deutsch](README.de.md) · **🇪🇸 Español** · [🇰🇷 한국어](README.ko.md) · [🇯🇵 日本語](README.ja.md)

Aplicación para ordenar las relaciones entre sujetos de fuentes abiertas, con seudonimización de datos personales mediante clave.

Estado: **desarrollo temprano** (versión preliminar). Implementación: Rust (núcleo) y React (ventana). El ucraniano es el idioma de referencia de la interfaz y la documentación.

ZaNytky es **software propietario**: un repositorio público no concede una licencia de código abierto (véanse LICENSE.md, COPYRIGHT.md, THIRD_PARTY_NOTICES.md). En el repositorio no se guardan datos de usuario.

> Esta traducción es automática y no ha sido revisada por hablantes nativos; el texto ucraniano es el de referencia.

## Qué hace

- Seudónimos unívocos de valores (nombres, números fiscales, teléfonos, correos) con una clave del sistema: el mismo valor da siempre el mismo código.
- Los originales solo se guardan sellados con la clave pública del investigador (age); solo el dueño de la clave privada puede abrirlos.
- Anonimización del texto antes del análisis por la IA y paquete para la IA (texto, instrucción, metadatos); decisiones del investigador sobre menciones no reconocidas.
- Comprobación de la respuesta de la IA, revisión humana, importación a la base (SQLite) como «sin verificar».
- Modelo: sujetos, fuentes con cadena hasta la fuente original, afirmaciones con valoración de fiabilidad y credibilidad, árbol de propiedad.
- Informe del investigador en Markdown, Word y PDF, protegido con una frase secreta.

## Descarga

Los archivos listos para Windows (zip portátil sin instalación, instaladores .msi/.exe) están en la página de [versiones](https://github.com/RomanZavadaM/ZaNytky/releases/latest). Las compilaciones no están firmadas: Windows puede avisar de un editor desconocido.

## Inicio rápido (aplicación)

1. Descomprima el archivo portátil y ejecute `ZaNytky.exe` (requiere Microsoft Edge WebView2, presente en Windows 10/11).
2. Pestaña «Configuración»: elija una carpeta de datos vacía y pulse «Preparar la carpeta»; cree una clave de investigador y guarde su archivo en un lugar seguro.
3. Pestaña «Preparar documento»: pegue el texto, pulse «Anonimizar y preparar el paquete para la IA», copie el texto y la instrucción y entréguelos a la IA.
4. Pestaña «Resultado de la IA»: pegue la respuesta (result.json), compruébela, lea la revisión e importe.
5. Pestañas «Base» e «Informe»: consulte las afirmaciones, confírmelas o refútelas, cree un informe.

## Línea de comandos

Lo mismo está disponible como comandos; `zanytky help` los lista todos.

```bash
zanytky init --home ./home
zanytky identity-new ./investigator.txt
zanytky register name "Іваненко Петро" --home ./home
zanytky prepare doc.md --home ./home --out ./pkg --id pkg-1 --title "Title" --identity ./investigator.txt
zanytky check  result.json --package pkg-1
zanytky import result.json --package pkg-1 --home ./home
zanytky report --home ./home --identity ./investigator.txt --out report.docx --format docx --passphrase-file ./pass.txt
```

## Seguridad

Guarde por separado la clave del sistema (`system.key` en la carpeta de datos), la clave privada del investigador y la frase secreta, y nunca las añada a un repositorio. La aplicación no envía nada por la red.

## Documentación

- [Guía de usuario](../user-guide/USER_GUIDE.es.md)
- [LICENSE](../../LICENSE.md)
