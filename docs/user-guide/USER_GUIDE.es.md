# ZaNytky - guía de usuario

[🇺🇦 Українська](USER_GUIDE.uk.md) · [🇬🇧 English](USER_GUIDE.en.md) · [🇫🇷 Français](USER_GUIDE.fr.md) · [🇩🇪 Deutsch](USER_GUIDE.de.md) · **🇪🇸 Español** · [🇰🇷 한국어](USER_GUIDE.ko.md) · [🇯🇵 日本語](USER_GUIDE.ja.md)

Esta guía describe la ventana de la aplicación. La versión ucraniana es el texto de referencia.

> Esta traducción es automática y no ha sido revisada por hablantes nativos; el texto ucraniano es el de referencia.

## 1. Configuración

- Elija una **carpeta de datos** (mejor vacía) y pulse «Preparar la carpeta». En ella aparecen la clave del sistema `system.key` y el archivo `recipients.txt` con las claves públicas de los investigadores.
- Cree una **clave de investigador**: la aplicación escribe la clave privada en un archivo que usted elige y añade la clave pública a `recipients.txt`. La clave privada hace falta para revelar nombres y para los informes; sin ella no se pueden revelar los datos. Guarde el archivo en un lugar seguro y haga una copia.

## 2. Preparar un documento

- Indique un título, el tipo de fuente (medios, registro, declaración de una autoridad, testimonio, rumor, propia), opcionalmente una dirección, y un id de paquete (único para cada documento).
- Pegue el texto o abra un archivo. Si hace falta, añada el **contexto del investigador**, por ejemplo «Ivanov I. dirige la empresa Ejemplo».
- Pulse «Anonimizar y preparar el paquete para la IA». La aplicación sustituye los nombres, números fiscales, teléfonos y correos conocidos por seudónimos. Copie el texto anonimizado y la instrucción y entréguelos a la IA.
- En «Menciones que el sistema no reconoció» elija para cada mención: **la misma persona** (indique el seudónimo), **persona nueva** (nombre completo, opcionalmente número fiscal, teléfono, correo) o **no es una persona**. Después vuelva a crear el paquete.

## 3. Resultado de la IA

- Pegue la respuesta de la IA (archivo result.json) y el id del paquete. Pulse «Comprobar»: la aplicación verifica la estructura y las reglas y muestra una revisión con citas. Solo después de la revisión pulse «Importar a la base de datos».
- Todo se importa como «sin verificar». Un paquete solo puede importarse una vez.

## 4. Base

- Lista de afirmaciones con una valoración (por ejemplo B3: fiabilidad de la fuente B, credibilidad 3) y un estado. Las personas se muestran solo por seudónimo.
- «Confirmar» solo es posible si existe una fuente que no sea un rumor ni un testimonio; de lo contrario la aplicación lo rechaza. «Refutar» marca la afirmación como refutada (no se elimina).

## 5. Informe

- Elija un formato (Word, PDF, Markdown) e introduzca una frase secreta. El informe revela a las personas con su clave privada, por lo que contiene datos personales abiertos. Sin frase secreta, el informe solo se crea tras confirmar expresamente «Sin frase secreta».
- Un informe protegido tiene la extensión `.age`; ábralo con [age](https://age-encryption.org) y su frase secreta.

## Seguridad

- Guarde por separado la clave del sistema, la clave privada del investigador y la frase secreta. Perder la clave privada significa que los datos ya no podrán revelarse.
- La aplicación no envía nada por la red. El texto que entrega a la IA sale de su ordenador, así que aporte solo material de fuentes abiertas.

## Preguntas frecuentes

- Windows avisa de un editor desconocido: la compilación no está firmada; elija «Más información» y «Ejecutar de todas formas».
- Ventana vacía o error al iniciar: compruebe que la carpeta `webview2` está junto a `ZaNytky.exe` (el archivo portátil no requiere instalación).

## Escenario de prueba (datos ficticios)

1. Pestaña «Configuración»: elija una carpeta vacía, pulse «Preparar la carpeta», cree una clave de investigador.
2. Pestaña «Preparar documento»: pegue el texto de `docs/examples/sample-document.md`, título «Documento de prueba», id `pkg-1`, pulse «Anonimizar y preparar el paquete».
3. En la lista de menciones no reconocidas elija «Persona nueva» para «Петро Іваненко», introduzca el nombre completo, el número fiscal 1234567890 y el teléfono +38 050 123 45 67, pulse «Aplicar». Vuelva a crear el paquete: ahora el nombre, el número fiscal y el teléfono se sustituyen por un solo seudónimo.
4. Copie la instrucción y el texto anonimizado en un chat de IA y pida que responda solo con JSON. Guarde la respuesta.
5. Pestaña «Resultado de la IA»: pegue la respuesta, pulse «Comprobar», lea la revisión y pulse «Importar a la base de datos».
6. Pestaña «Base»: consulte las afirmaciones; pruebe «Confirmar» (para una afirmación cuya fuente es un rumor, la aplicación lo rechaza).
7. Pestaña «Informe»: cree un informe Word con frase secreta; ábralo con age y compruebe que contiene el nombre real.
