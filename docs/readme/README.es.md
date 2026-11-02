# ZaNytky («За нитку»)

[🇺🇦 Українська](../../README.md) · [🇬🇧 English](README.en.md) · [🇫🇷 Français](README.fr.md) · [🇩🇪 Deutsch](README.de.md) · **🇪🇸 Español** · [🇰🇷 한국어](README.ko.md) · [🇯🇵 日本語](README.ja.md)

## Descarga

**Última versión: [ZaNytky v0.0.8](https://github.com/RomanZavadaM/ZaNytky/releases/tag/v0.0.17) (versión preliminar)**

| Plataforma | Archivo |
|---|---|
| Windows - instalador (recomendado) | [ZaNytky_0.0.17_Setup_Windows_x64.exe](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.17/ZaNytky_0.0.17_Setup_Windows_x64.exe) |
| Windows - MSI | [ZaNytky_0.0.17_Windows_x64.msi](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.17/ZaNytky_0.0.17_Windows_x64.msi) |
| Windows - archivo portátil (solo para conocer) | [ZaNytky_0.0.17_Windows_x64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.17/ZaNytky_0.0.17_Windows_x64_Portable.zip) |
| Windows - línea de comandos | [ZaNytky_0.0.17_CLI_Windows_x64.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.17/ZaNytky_0.0.17_CLI_Windows_x64.zip) |
| macOS (Apple Silicon) | [ZaNytky_0.0.17_macOS_arm64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.17/ZaNytky_0.0.17_macOS_arm64_Portable.zip) |
| macOS (Intel) | [ZaNytky_0.0.17_macOS_x86_64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.17/ZaNytky_0.0.17_macOS_x86_64_Portable.zip) |

- Sumas de comprobación: [SHA256SUMS.txt](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.17/SHA256SUMS.txt) y [página de la versión](https://github.com/RomanZavadaM/ZaNytky/releases/tag/v0.0.17). Todas las versiones: [Releases](https://github.com/RomanZavadaM/ZaNytky/releases).
- Las compilaciones no están firmadas: Windows puede avisar de un editor desconocido; las de macOS no están notarizadas ni probadas en un Mac real.
- El archivo portátil es solo para conocer el programa, sin ninguna garantía; para el uso habitual, use el instalador.

Aplicación para ordenar los vínculos entre sujetos a partir de fuentes abiertas: fuentes con texto completo y archivos, afirmaciones con citas, grafo de vínculos, búsqueda del camino más corto entre sujetos. Los datos personales se sustituyen por seudónimos con clave.

Estado: **desarrollo temprano** (versión preliminar 0.0.8). ZaNytky es **software propietario**: un repositorio público no concede licencia de código abierto (LICENSE.md, COPYRIGHT.md, THIRD_PARTY_NOTICES.md). En el repositorio no se guardan datos de usuarios.

> Esta traducción es automática y no ha sido revisada por hablantes nativos; prevalece el texto ucraniano.

## Qué hace

- **Fuentes:** texto completo y archivos (escaneos, fotos, vídeo) en la base local; búsqueda por título, texto y dirección.
- **Importación de un canal de Telegram** (exportación de Telegram Desktop, JSON): cada publicación pasa a ser una fuente con texto y adjuntos.
- **Afirmaciones con cita** de la fuente: tipo, partes con roles (sujetos existentes o nuevos), nivel de credibilidad; siempre «sin verificar».
- **Sujetos:** ficha con afirmaciones, fuentes, cadena de propiedad y rasgos comunes (dirección, teléfono, matrícula, cuenta); fusión de duplicados; grafo; camino más corto entre dos sujetos.
- **Seudonimización:** códigos unívocos con clave del sistema; originales solo sellados (age) para la clave del investigador; anonimización del texto y paquete para la IA; control e importación de la respuesta.
- **Informe** en Markdown, Word y PDF con frase de contraseña; copia cifrada de la carpeta de datos. 7 idiomas.

## Inicio rápido

1. Instale la aplicación (o descomprima el archivo portátil y ejecute `ZaNytky.exe`).
2. «Ajustes»: elija una carpeta de datos vacía fuera de la carpeta del código, «Preparar carpeta», cree la clave del investigador y guárdela aparte.
3. «Fuentes»: importe la exportación de un canal de Telegram (botón superior) o añada texto y archivos a mano.
4. Abra una fuente y cree una afirmación: cita, partes, roles. Consulte «Sujetos» y «Grafo».
5. Para el análisis con IA: «Documento» (anonimización y paquete) y «Resultado IA» (control e importación). «Informe» da el resumen.

## Línea de comandos

Lo mismo está disponible como comandos; `zanytky help` los muestra todos.

```bash
zanytky init --home ./home
zanytky identity-new ./investigator.txt
zanytky import-telegram "ChatExport_2026-11-01" --home ./home --reliability F
zanytky import result.json --package pkg-1 --home ./home
zanytky report --home ./home --identity ./investigator.txt --out report.docx --format docx --passphrase-file ./pass.txt
zanytky backup --home ./home --out copy.znbak --passphrase-file ./pass.txt
```

## Principios

- Cada afirmación tiene una fuente y una cadena hasta la fuente original; un rumor no se vuelve hecho, y los ecos de un rumor son una sola fuente original.
- Sin etiquetas valorativas sobre personas: se muestran origen y fiabilidad, no veredictos.
- Solo fuentes abiertas. Textos y archivos se guardan localmente, sin anonimizar; la IA recibe solo texto anonimizado.

## Limitaciones de la versión actual (0.0.17)

- Versión preliminar: los instaladores no están firmados.
- Aún no hay importación de PDF ni reconocimiento de texto en escaneos o vídeo; la cita de una afirmación se pega a mano.
- El informe Word y la vista PDF no se han comprobado en Microsoft Word ni en visores PDF; el reconocimiento de nombres desconocidos es sencillo.
- Mantenga la carpeta de datos fuera de la carpeta del código y no ponga datos personales en el identificador del paquete: llega a la IA sin anonimizar.

## Seguridad

Guarde por separado la clave del sistema (`system.key` en la carpeta de datos), la clave privada del investigador y la frase de contraseña, y nunca las añada al repositorio. La aplicación no envía nada a la red.

## Documentación

- [Guía del usuario](../../docs/user-guide/USER_GUIDE.es.md)
- [Notas de versión](../../docs/releases)
- [LICENSE](../../LICENSE.md)
