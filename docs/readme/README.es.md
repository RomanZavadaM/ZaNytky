# ZaNytky («За нитку»)

[🇺🇦 Українська](../../README.md) · [🇬🇧 English](README.en.md) · [🇫🇷 Français](README.fr.md) · [🇩🇪 Deutsch](README.de.md) · **🇪🇸 Español** · [🇰🇷 한국어](README.ko.md) · [🇯🇵 日本語](README.ja.md)

## Descarga

**Última versión: [ZaNytky v0.0.27](https://github.com/RomanZavadaM/ZaNytky/releases/tag/v0.0.27) (versión preliminar)**

| Plataforma | Archivo |
|---|---|
| Windows - instalador (recomendado) | [ZaNytky_0.0.27_Setup_Windows_x64.exe](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.27/ZaNytky_0.0.27_Setup_Windows_x64.exe) |
| Windows - MSI | [ZaNytky_0.0.27_Windows_x64.msi](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.27/ZaNytky_0.0.27_Windows_x64.msi) |
| Windows - archivo portátil (solo para conocer) | [ZaNytky_0.0.27_Windows_x64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.27/ZaNytky_0.0.27_Windows_x64_Portable.zip) |
| Windows - línea de comandos | [ZaNytky_0.0.27_CLI_Windows_x64.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.27/ZaNytky_0.0.27_CLI_Windows_x64.zip) |
| macOS (Apple Silicon) | [ZaNytky_0.0.27_macOS_arm64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.27/ZaNytky_0.0.27_macOS_arm64_Portable.zip) |
| macOS (Intel) | [ZaNytky_0.0.27_macOS_x86_64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.27/ZaNytky_0.0.27_macOS_x86_64_Portable.zip) |

- Sumas de comprobación: [SHA256SUMS.txt](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.27/SHA256SUMS.txt) y [página de la versión](https://github.com/RomanZavadaM/ZaNytky/releases/tag/v0.0.27). Todas las versiones: [Releases](https://github.com/RomanZavadaM/ZaNytky/releases).
- Las compilaciones no están firmadas: Windows puede avisar de un editor desconocido; las de macOS no están notarizadas ni probadas en un Mac real.
- El archivo portátil es solo para conocer el programa, sin ninguna garantía; para el uso habitual, use el instalador.

Aplicación para ordenar los vínculos entre sujetos a partir de fuentes abiertas: fuentes con texto completo y archivos, afirmaciones con citas, grafo de vínculos, búsqueda del camino más corto entre sujetos. Los datos personales se sustituyen por seudónimos con clave.

Estado: **desarrollo temprano** (versión preliminar 0.0.23). ZaNytky es **software propietario**: un repositorio público no concede licencia de código abierto (LICENSE.md, COPYRIGHT.md, THIRD_PARTY_NOTICES.md). En el repositorio no se guardan datos de usuarios.

> Esta traducción es automática y no ha sido revisada por hablantes nativos; prevalece el texto ucraniano.

## Qué hace

- **Fuentes:** texto completo y archivos (escaneos, fotos, vídeo) en la base local; búsqueda por título, texto y dirección.
- **Importación de un canal de Telegram** (exportación de Telegram Desktop, JSON): cada publicación pasa a ser una fuente con texto y adjuntos.
- **Afirmaciones con cita** de la fuente: tipo, partes con roles (sujetos existentes o nuevos), nivel de credibilidad; siempre «sin verificar».
- **Sujetos:** ficha con afirmaciones, fuentes, cadena de propiedad y rasgos comunes (dirección, teléfono, matrícula, cuenta); fusión de duplicados; grafo; camino más corto entre dos sujetos.
- **Seudonimización:** códigos unívocos con clave del sistema; originales solo sellados (age) para la clave del investigador; anonimización del texto y paquete para la IA; control e importación de la respuesta.
- **Informe** en Markdown, Word y PDF con frase de contraseña; copia cifrada de la carpeta de datos. 7 idiomas.
- **Pruebas y verificación:** a una afirmación se le adjunta un documento («Añadir prueba»: registro, declaración oficial, medios, testimonio, propio); «confirmado» solo lo fija un documento de registro o de declaración oficial, cualquier tipo puede refutar.
- **Análisis de la base:** búsqueda rápida en las fuentes, el resumen, pares de sujetos en un mismo párrafo, coincidencias entre canales, descubrimiento automático de sujetos; el grafo (entero o en parte) se guarda como un archivo HTML autónomo.
- **Línea de comandos para el análisis automatizado** (search, source, discover, channels, overlaps, pairs, entity, statements, add-entity, add-source, make-statements, evidence, detach, delete-statement, delete-source, merge, unmerge, graph): lectura con `--json`; `make-statements` escribe un lote de afirmaciones solo si la cita de cada una está en el texto de su fuente; los errores se corrigen con comandos, no a mano.

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

Análisis y corrección de errores por comandos:

```bash
zanytky search "Приклад" --home ./home
zanytky source 123 --full --home ./home
zanytky discover --source 123 --home ./home
zanytky make-statements --file statements.json --home ./home
zanytky evidence --statement 5 --kind registry --title "Витяг" --url https://example.org/x --status confirmed --home ./home
zanytky graph --out graph.html --entity 32 --depth 2 --home ./home
```

`--expect` en `evidence` impide adjuntar una prueba a una afirmación equivocada; `detach`, `delete-statement`, `delete-source` y `merge` corrigen errores (borrar es irreversible, haga `backup` antes de cambios masivos).

## Principios

- Cada afirmación tiene una fuente y una cadena hasta la fuente original; un rumor no se vuelve hecho, y los ecos de un rumor son una sola fuente original.
- Sin etiquetas valorativas sobre personas: se muestran origen y fiabilidad, no veredictos.
- Solo fuentes abiertas. Textos y archivos se guardan localmente, sin anonimizar; la IA recibe solo texto anonimizado.

## Limitaciones de la versión actual (0.0.27)

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
