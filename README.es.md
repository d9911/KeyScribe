# Keyscribe

<img src="assets/icons/keyscribe.svg" alt="Keyscribe icon" width="112">

[English](README.md) · [Русский](README.ru.md) · [Español](README.es.md)

![Rust 2021](https://img.shields.io/badge/Rust-2021-000000?logo=rust&logoColor=white)
![egui 0.33](https://img.shields.io/badge/UI-egui_0.33-4472C4)
![macOS Apple Silicon](https://img.shields.io/badge/macOS-Apple_Silicon-333333?logo=apple&logoColor=white)
![Windows x64](https://img.shields.io/badge/Windows-x64-0078D4)

Texto, ratón y ciclos de acciones · Inglés y ruso combinados · macOS y Windows

<details>
<summary>Contenido</summary>

[Funciones](#funciones) · [Tecnologías](#tecnologías) · [Inicio rápido](#inicio-rápido) · [Distribuciones y caracteres](#distribuciones-y-caracteres) · [Movimiento aleatorio del ratón](#movimiento-aleatorio-del-ratón) · [Ciclos de acciones](#ciclos-de-acciones) · [Citrix en Mac](#citrix-en-mac) · [Comandos Make](#comandos-make) · [Compilar para macOS](#compilar-para-macos) · [Compilar para Windows](#compilar-para-windows) · [Desarrollo y verificación](#desarrollo-y-verificación) · [Autor](#autor) · [Licencia](#licencia)

</details>

![Keyscribe en macOS: texto mixto y cambio de distribución configurable](assets/screenshots/keyscribe-light.jpg)

**Pega aquí. Escribe en la ventana que elijas.** Keyscribe convierte texto en
pulsaciones físicas con pausas configurables. La captura muestra la aplicación
actual en macOS con texto de demostración; no se inició la escritura remota para
la captura. La interfaz está actualmente en ruso; la documentación tiene tres idiomas.

Elige **Светлая** (Claro) o **Тёмная** (Oscuro) en la barra superior. El editor y los ajustes aparecen en columnas en una ventana ancha y uno debajo del otro en una estrecha. La selección del tema dura hasta cerrar la aplicación.

<details>
<summary>Tema oscuro</summary>

![Keyscribe con el tema verde oscuro](assets/screenshots/keyscribe-dark.jpg)

</details>

<details>
<summary>Movimiento aleatorio del ratón</summary>

![Ajustes del ratón](assets/screenshots/keyscribe-mouse.jpg)

</details>

<details>
<summary>Editor del ciclo de acciones</summary>

![Ajustes del ciclo](assets/screenshots/keyscribe-cycle.jpg)

</details>

## Funciones

| Función          | Comportamiento                                                                    |
| ---------------- | --------------------------------------------------------------------------------- |
| Texto mixto      | English US y ruso ЙЦУКЕН, mayúsculas, minúsculas, números, ASCII imprimible y `№` |
| Cambio de idioma | Alt+Shift, Ctrl+Shift, Win+Space o una combinación personalizada                  |
| Líneas y sangría | Pulsaciones físicas de Enter y Tab; conserva los espacios                         |
| Velocidad        | Cuenta atrás, duración de pulsación y pausas entre teclas y cambios de idioma     |
| Validación       | Comprueba todo el texto antes de enviar la primera tecla                          |
| Parada           | Ctrl+Alt+F12, botón de parada o regreso a Keyscribe                               |
| Uso local        | Texto en memoria, sin servidor ni instalación dentro de Citrix                    |
| Ratón aleatorio  | Destinos, curvas, duración y pausas aleatorios en una zona limitada               |
| Ciclos           | Teclas, combinaciones, clics, movimientos y esperas con repeticiones              |
| Apariencia       | Tema claro crema y tema verde oscuro; navegación y controles siempre visibles     |

## Tecnologías

| Componente             | Implementación                                                 |
| ---------------------- | -------------------------------------------------------------- |
| Lenguaje               | Rust 2021; compilador mínimo declarado en Cargo.toml: 1.88     |
| Interfaz               | egui / eframe 0.33, OpenGL mediante glow                       |
| Teclado                | Enigo 0.6, códigos físicos de teclas                           |
| Atajo global de parada | global-hotkey 0.9                                              |
| Ejecución              | Hilo independiente, esperas cancelables y liberación de teclas |
| Compilación            | Cargo, Cargo.lock y scripts para cada plataforma               |

Se ejecuta localmente en macOS Apple Silicon (M1 y posteriores) o Windows x64.
No necesita JavaScript, un backend ni conexión de red de la aplicación. El texto
y los ajustes se mantienen en memoria y no se guardan al cerrar. El texto no se
registra ni se transmite. El portapapeles solo se utiliza al pegar en el editor
local; Keyscribe no pega en la aplicación de destino.

## Inicio rápido

Si falta una opción obligatoria, al pulsar iniciar se desplaza la vista hasta ella
y aparece un borde rojo redondeado con una explicación. El borde pulsa tres veces
y permanece hasta completar el campo; después señala la siguiente opción pendiente.
Continuar en el diálogo de permisos también señala el siguiente paso. Completar
los campos nunca inicia acciones automáticamente.

1. Abre la aplicación compilada siguiendo las instrucciones de tu plataforma.
2. Pega el texto en Keyscribe con Cmd+V o Ctrl+V.
3. Selecciona **dónde se ejecuta Keyscribe**: macOS / Citrix desde Mac o Windows /
   Citrix desde Windows. El perfil Mac propone **Control+Option+Space** y Windows
   **Win+Space**; también hay **Ctrl+Win+Space**, Alt+Shift, Ctrl+Shift y atajos
   personalizados. Comprueba el atajo dentro de tu sesión remota.
   Después indica la **distribución actual de Windows dentro de Citrix**: English US
   o ruso ЙЦУКЕН estándar. Para texto mixto, activa el cambio automático y
   configura la combinación que cambia el idioma dentro de la sesión remota.
4. Selecciona la misma distribución en Citrix y desactiva Bloq Mayús. Desactiva
   la sangría automática, el cierre de paréntesis y las comillas inteligentes
   del editor si necesitas conservar el texto exacto.
5. Marca la confirmación, pulsa **Начать набор** (Iniciar escritura) y selecciona
   el campo de destino antes de terminar la cuenta atrás. No mantengas pulsados
   modificadores ni cambies de ventana durante la escritura.
6. Detén con **Ctrl+Alt+F12**, el botón de parada o regresando a Keyscribe.
   En Mac: Control+Option+F12; añade Fn si F12 controla funciones multimedia.
   Comprueba lo ya escrito antes de volver a empezar: no hay reanudación automática.

Valores iniciales: cuenta atrás de 5 segundos, pulsación de 25 ms, pausa entre
teclas de 100 ms y pausa tras cambiar de idioma de 400 ms. Son ajustables por separado.

## Distribuciones y caracteres

El cambio automático requiere **exactamente dos distribuciones remotas**:
English US y ruso ЙЦУКЕН estándar. Indica la distribución inicial correcta antes
de cada ejecución. El atajo debe alternar entre ambas, no seleccionar siempre
un único idioma. Puedes elegir Control+Option+Space, Win+Space, Ctrl+Win+Space,
Alt+Shift, Ctrl+Shift o combinar
Ctrl/Alt/Shift/Win con Space, un número o F1–F12. Desactiva el cambio automático
si vas a usar una sola distribución.

Se conserva el idioma actual para caracteres que puede producir. Tras terminar
o cancelar, la distribución queda en su último estado; compruébala antes de
otra ejecución. Aumenta la pausa de cambio si la sesión necesita más tiempo.

El entorno y el idioma empiezan sin seleccionar; el texto y los ciclos requieren
ambos. Después de ejecutar, cancelar o fallar el inicio de sesión, debes indicar
el idioma de nuevo. Cambiar el entorno borra el idioma y aplica el atajo de ese
perfil. Los perfiles no cambian los ajustes del sistema ni añaden compatibilidad
con el ruso de macOS: la distribución rusa de destino sigue siendo Windows ЙЦУКЕН.

Cada salto de línea se envía como Enter, incluido el salto final. CRLF y CR
aislado se convierten en un solo Enter. No se añade un Enter extra. Tab es una
tecla real: puede cambiar de campo o activar el autocompletado. Usa espacios para
la sangría si el editor interpreta Tab de otra forma.

Todo el texto se valida antes de empezar. Los caracteres no admitidos, como emoji,
espacios inseparables y letras de otros alfabetos, generan un error con línea y
columna; no se sustituyen silenciosamente. No se admite Unicode arbitrario.
El contador muestra teclas de texto enviadas, sin contar los atajos de idioma;
no confirma su recepción en el destino.

Prueba el [ejemplo mixto](samples/mixed.txt), [inglés](samples/us.txt) o
[ruso](samples/ru.txt) en un editor remoto vacío antes de una ejecución larga.
Comprueba mayúsculas, signos, espacios, Enter, cambio de idioma y parada de emergencia.

## Movimiento aleatorio del ratón

Abre **Движение мыши** (Ratón aleatorio). Ajusta los radios X/Y (5–2000 unidades
de coordenadas), las pausas y la duración, o elige continuar hasta detenerlo.
Valores iniciales: radios 120/80, pausas de 200–1800 ms y temporizador de 60 segundos.

La zona se fija alrededor del cursor al terminar la cuenta atrás: sitúalo dentro
del área deseada antes de empezar. Esta versión admite la pantalla principal
y limita la zona a sus bordes. Este modo no hace clics. Si mueves el cursor fuera
de la zona, se detiene al comprobarlo antes del siguiente movimiento; volver a
Keyscribe también lo detiene.

El generador del sistema operativo elige nuevos destinos, dos puntos de control,
duraciones de 250–1400 ms y pausas. No hay una trayectoria grabada; la selección
aleatoria no garantiza que todos los movimientos sean únicos.

## Ciclos de acciones

Abre **Цикл действий** (Ciclo de acciones) y añade pasos:

- **Teclas:** Enter, Tab, Escape, Backspace, Delete, flechas, Home/End,
  PageUp/PageDown, Space, A–Z, números y F1–F12, con Ctrl/Alt/Shift/Win opcionales.
- **Clic:** X/Y de la pantalla principal, botón izquierdo/derecho/central, simple o doble.
- **Movimiento:** desplazamiento a coordenadas concretas.
- **Espera:** entre 10 ms y una hora.

Puedes reordenar, duplicar y borrar pasos. Se admiten hasta 200 pasos y 1–10000
repeticiones, con una pausa independiente entre repeticiones. Para capturar
coordenadas, activa el botón **Ctrl+Alt+F11** del paso, apunta al destino en la
ventana externa y pulsa ese atajo. También puedes usar **Захват через 3 с**
(Capturar en 3 segundos) y situar el cursor antes de terminar el temporizador.
Las coordenadas se pueden introducir a mano. En Mac, las teclas F pueden requerir Fn.

Comprueba una repetición antes de aumentar la cantidad. Todos los pasos y las
coordenadas se validan antes de la primera acción. Ctrl+Alt+F11/F12 están reservados.
Las letras representan teclas físicas US y dependen de la distribución activa
en el destino; no usan el cambio automático del modo de texto.

Todos los modos comparten la cuenta atrás, **Ctrl+Alt+F12**, el botón de parada
y la parada al volver a Keyscribe. Solo se ejecuta un modo a la vez. No se repiten
acciones automáticamente tras un error o cancelación. El contador muestra acciones
enviadas, no tareas cerradas con éxito. No se reconoce la pantalla ni el resultado
de las operaciones: añade las esperas necesarias. Los ajustes y el ciclo se
mantienen únicamente hasta cerrar la aplicación.

## Citrix en Mac

Selecciona **Keyboard input mode → Scancode**. Para este modo Citrix recomienda
**Don't sync** en la sincronización de distribuciones. Elige el idioma inicial
dentro de la sesión y vuelve a conectarte tras cambiar estas preferencias.
El modo Automatic puede seleccionar Unicode y procesar las distribuciones de otra forma.

Para Win+Space, activa el envío de Windows logo key mediante **Command (right)**:
Keyscribe envía Win como Command derecho en Mac. Comprueba que el atajo cambia
el idioma remoto y no el del ordenador local. Las opciones pueden variar en
clientes de navegador y otras versiones de Citrix.

Fuentes: [Teclado de Citrix](https://docs.citrix.com/us/en/citrix-workspace-app-for-mac/devices/keyboard.html),
[Idiomas y distribuciones de Windows](https://support.microsoft.com/en-us/windows/hardware/input-devices/manage-the-language-and-keyboard-input-layout-settings-in-windows).

**No se ha verificado la exactitud completa en una sesión concreta de Citrix.**
Keyscribe no lee la pantalla remota ni detecta eventos perdidos, una distribución
incorrecta, el cambio a otra ventana externa o transformaciones del editor.
Comprueba también la parada global en Citrix a pantalla completa. Si faltan
caracteres, aumenta las pausas y la duración de pulsación.

## Comandos Make

Ejecuta los comandos desde `keyscribe/`. GNU Make y los requisitos de compilación
de la plataforma indicados abajo deben estar en `PATH`. En Windows, instala GNU
Make por separado y usa el entorno de compilación MSVC. El Makefile utiliza Windows
PowerShell para empaquetar en Windows y el script de shell en macOS.

| Comando | Acción |
| --- | --- |
| `make` o `make dev` | Compilar y ejecutar la versión de desarrollo |
| `make build` | Crear la aplicación y el ZIP de distribución para el sistema actual |
| `make run` | Abrir la compilación existente sin recompilar |
| `make start` | Compilar la aplicación de distribución y abrirla |
| `make check` | Comprobar el formato, ejecutar Clippy y las pruebas |
| `make test` | Ejecutar las pruebas |
| `make lint` | Ejecutar Clippy con advertencias tratadas como errores |
| `make fmt` | Formatear el código Rust |
| `make fmt-check` | Comprobar el formato sin modificar archivos |
| `make help` | Mostrar los comandos disponibles |

Si no hay una compilación, `make run` lo indica en lugar de compilar. En macOS
abre `dist/Keyscribe.app`; en Windows, `dist/windows-x64/Keyscribe.exe`.
Usa `make start` para compilar y abrir la aplicación empaquetada con un solo comando.
Cierra otra instancia de Keyscribe antes de iniciar: puede tener registrados los
atajos globales. Abrir la aplicación no inicia la escritura ni las acciones del ratón.

## Compilar para macOS

Desde el directorio del código fuente, con Rust/rustup y Xcode Command Line Tools:

```sh
sh scripts/build-macos.sh
open dist/Keyscribe.app
```

Resultados: `dist/Keyscribe.app` y `dist/Keyscribe-macos-arm64.zip`.
Sin permiso macOS aparece un diálogo bloqueante. Comprueba Accesibilidad y abre
una conexión de Enigo sin enviar eventos; la app no exige otro permiso de envío.
Tiene botones para solicitar acceso, abrir ajustes, comprobar de
nuevo y mostrar la copia actual en Finder. Activa Keyscribe en **Ajustes del
Sistema → Privacidad y seguridad → Accesibilidad**. Se comprueba automáticamente,
al regresar y antes de iniciar. Continuar vuelve a comprobar el acceso; si falta,
el diálogo permanece abierto y resalta el bloque de permisos. Nunca inicia acciones automáticamente. Puedes abrirlo de nuevo desde
**Доступ macOS** en la cabecera. Revocar el permiso cancela la tarea activa.
Si macOS no muestra otra solicitud, abre los ajustes con el botón del diálogo.

Cierra Keyscribe antes de recompilar: el script no sustituye una app en ejecución.
Si el acceso falla después de mover o recompilar aunque el interruptor esté activo,
pulsa **Сбросить старое разрешение…** en el diálogo y confirma. Keyscribe registra
su ubicación actual en LaunchServices y ejecuta únicamente
`/usr/bin/tccutil reset Accessibility local.keyscribe.desktop`. No cambia permisos
de otras aplicaciones. Solicita acceso de nuevo y abre los ajustes. Activa la copia
actual y reiníciala; si falta la entrada, añade su `.app` con `+`. El diálogo muestra
la ruta y el botón de Finder. También puedes eliminar la entrada antigua y añadir
la actual manualmente. El restablecimiento exige confirmación explícita y nunca
concede acceso automáticamente. Con
`cargo run`, el permiso puede pertenecer al terminal o al ejecutable; para el uso
habitual abre el `.app` desde una ubicación estable. El requisito de la firma ad
hoc predeterminada depende del hash y puede cambiar al recompilar. Si dispones
de un certificado estable, usa `KEYSCRIBE_SIGNING_IDENTITY="nombre del certificado" make build`.
Cambiar el certificado también puede requerir un permiso nuevo. No se realiza
la notarización de Apple. Fuentes: [solicitud de Accesibilidad de Apple](https://developer.apple.com/documentation/applicationservices/1459186-axisprocesstrustedwithoptions),
[identidad de firma de Apple](https://developer.apple.com/documentation/technotes/tn3127-inside-code-signing-requirements).

## Compilar para Windows

Compila en Windows con Rust stable y Visual Studio Build Tools, incluyendo
**Desktop development with C++** y Windows SDK:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build-windows.ps1
```

El icono del EXE se incorpora mediante `build.rs` como [recurso ICON](https://learn.microsoft.com/en-us/windows/win32/menurc/icon-resource).
Compila desde Developer PowerShell for Visual Studio con `rc.exe` de Windows SDK
en `PATH`. La comprobación de código Windows desde macOS omite este recurso.

Resultados: `dist/windows-x64/Keyscribe.exe` y `dist/Keyscribe-windows-x64.zip`.
Ejecuta la aplicación en el ordenador local junto a Citrix. No necesita instalarse
en el escritorio remoto. Las restricciones sobre procesos elevados se describen
en [SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput).

## Desarrollo y verificación

Desde el código fuente con Rust instalado:

```sh
cargo run --locked
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
```

| Módulo                 | Responsabilidad                                               |
| ---------------------- | ------------------------------------------------------------- |
| `src/keyboard.rs`      | Validación y planificación de teclas y cambios de idioma      |
| `src/shortcut.rs`      | Combinación configurable de cambio de idioma                  |
| `src/playback.rs`      | Emisión y liberación de teclas y modificadores                |
| `src/desktop.rs`       | Escritura en un hilo independiente                            |
| `src/main.rs`          | Interfaz y atajo global de parada                             |
| `src/cycle.rs`         | Pasos, repeticiones finitas y cancelación sin reintentos      |
| `src/motion.rs`        | Trayectorias limitadas con aleatoriedad del sistema operativo |
| `src/workspace_ui.rs` | Distribución de la ventana y controles de ejecución fijos |
| `src/appearance.rs` | Paletas clara y oscura, tipografía y estilos de componentes |
| `src/automation_ui.rs` | Editor del ciclo, captura de coordenadas y ajustes del ratón  |

Las dependencias están fijadas en `Cargo.lock`. Consulta los códigos físicos en
[Enigo Keyboard::raw](https://docs.rs/enigo/0.6.1/enigo/trait.Keyboard.html#tymethod.raw).
Se ha generado una compilación local para macOS arm64. Para Windows x64 se ha
comprobado la compilación del código y se incluye un script; no se ha generado ni
ejecutado un `.exe` en Windows. Esto no verifica una sesión específica de Citrix.

## Fuente del icono

[keyscribe.svg](assets/icons/keyscribe.svg) es la fuente vectorial, sin imágenes
incrustadas ni fuentes del sistema. Conserva la K y el cursor sin la textura
ráster. `keyscribe-128.png`, `keyscribe.icns` y `keyscribe.ico` se generan para
la interfaz y el empaquetado. Para actualizarlos, usa
`node scripts/generate-icons.mjs /ruta/a/node_modules/sharp` con Node.js y `sharp`
instalado por separado (verificado con 0.35.5). La compilación normal y la app no
necesitan Node.js. `.DS_Store`, Finder `Icon` e `Icon` terminado en CR se ignoran
en Git a cualquier profundidad y se excluyen de los paquetes. Conserva el icono
local de la carpeta.

## Autor

Denis Gutsuliak · [d9911.org](https://d9911.org).

## Licencia

Consulta las condiciones completas en [LICENSE](LICENSE).

[![License](https://img.shields.io/badge/license-see_LICENSE-blue)](LICENSE)


## Instaladores y Ubuntu X11

Comandos: `make setup`, `make doctor`, `make local`; PowerShell sin GNU Make. Consulte [INSTALLERS.md](docs/INSTALLERS.md) para DMG, Windows setup, DEB, atajos de emergencia de Linux y límites de verificación.
