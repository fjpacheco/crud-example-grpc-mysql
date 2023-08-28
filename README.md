[![Rust](https://github.com/fjpacheco/crud-example-grpc-mysql/actions/workflows/rust.yml/badge.svg)](https://github.com/fjpacheco/crud-example-grpc-mysql/actions/workflows/rust.yml)

# CRUD con gRPC y MySQL en Rust 

Prueba técnica donde se desarrolla una aplicación sencilla en Rust que permite a los usuarios interactuar con una base de datos MySQL, utilizando gRPC para la comunicación.

## Entorno de desarrollo 

Se desarrolló en el lenguaje de Rust, utilizando principalmente el crate de [sqlx](https://crates.io/crates/sqlx) para el manejo de la base de datos MySQL, y el crate [tonic](https://crates.io/crates/tonic) para el mandejo del cliente y servidor gRPC. Para facilitar el desarrollo se utilizó los contenedores de Docker para la base de datos MySQL evitando el uso de una base de datos remota.

## Acceso a la base de datos

Para la ejecución y buildeo de la **base de datos de desarrollo** mediante docker, específicamente con docker-compose (por simplicidad de ejecución de comandos, no había necesidad de "orquestar" N servicios con docker-compose):

```bash
docker-compose up --build
```

Y adicionalmente para parar la ejecución de la base de datos se ejecuta:

```bash
docker-compose down
```

Para utilizar la **base de datos de producción**, se debe solicitar la contraseña y host al desarrollador para actualizar el archivo [.env](.env) con las variables de entorno correspondientes. De esta forma se podrá ejecutar la base de datos MySQL provista por Google Cloud. 

## Ejecución de entorno de pruebas 

Para ejecutar el entorno de pruebas, se necesita tener la base de datos local con docker ejecutándose (¡queda pendiente resolverlo con [mocks](https://asomers.github.io/mock_shootout/) en Rust para evitar usar una conexión real a una base de datos en los tests!) y ejecutar el siguiente comando mediante cargo:

```bash
cargo test
```
O mediante el archivo makefile donde ya levanta la MySQL, ejecuta los tests y apaga la MySQL:

```bash
make test
```
Precaución: se necesita que la base de datos de desarrollo esté vacía por si hay repetición de ids en los tests con ids existentes en la tabla de usuarios (¡esto se resolvería con mocks en Rust!), el cliente vía CLI puede ayudar a resetear la tabla de usuarios mediante el comando `reset-table`.

Se testea tanto la capa de acceso de datos como la capa de servicio con los endpoints vía gRPC.

Para ambos tipos de testeos, hay que tener en cuenta que se necesita tener la base de datos de desarrollo ejecutándose y además que Rust ejecuta los tests en diferentes hilos (con una cfg en cargo se podría ejecutar en un solo hilo, pero busco rapidez al ejecutar tests). Para ello, antes de ejecutar los tests, habrá un thread encargado de "limpiar" la base de datos para que en próximos tests no haya problemas de ids repetidos. En [data/handler.rs](src/data/handler.rs) se detalla más sobre esto y como se atacó dicha problemática.

Adicionalmente en cada push a main se ejecuta un pipeline de Github Actions que ejecuta los tests (que a su vez levanta una base de datos de desarrollo con Docker Hub de GitHub Actions), como también ejecuta los linters (format & clippy) y el build. 

## Acerca del registro de actividad 

Se utilizó el crate [log](https://crates.io/crates/log) para el manejo de logs del sistema. Actualmente todo tipo de log se muestra en el _stdout_ del servidor y se visualizan las operaciones Info del acceso a la base de datos como el tipo de operación que realiza algún cliente. El nivel inicial de log es de Info, pero se puede modificar en el [lib.rs](src/lib.rs) o sino ejecutando el servidor con la variable de entorno RUST_LOG=debug. 

Con RUST_LOG=debug se podrá observar en qué thread id del runtime de tokio se ejecuta la petición al servidor gRPC. Esto es útil para ver que en el servidor gRPC se ejecutan operaciones de forma concurrente (y al usar una base de datos relacional, nos garantiza ACID para estas operaciones concurrentes). Este efecto se puede observar en mayor detalle cuando se ejecuta el servidor con múltiples clientes simulados.

## Acerca del manejo de errores

Los errores se manejan mediante el uso de Results en Rust, como el operador ? para propagar errores. Los errores de la base de datos se manejan en el archivo [errors.rs](src/errors.rs) el cual se encarga de convertir (mediante el trait From) cada error del crate sqlx a un error del negocio (ErrorKinsper). A su vez cada error del negocio, en ese mismo archivo se convierte a un error de gRPC (Status) para ser enviado al cliente. De esta forma ganamos un manejo de errores más robusto y mantenible, ubicando el manejo de errores en un solo lugar mediante el uso de las características de Rust.

## Manual de Uso

### Ejecucion del Servidor

Para la ejecución del servidor mediante cargo:

```bash
cargo run --bin server
```

Previamente hay que tener en [.env](.env) las variables de entorno correspondientes para la conexión a la base de datos (sea así de desarrollo o producción).

### Servidor con Multiples Clientes 

Teniendo el servidor ejecutado, se puede ejecutar múltiples clientes mediante el siguiente comando:

```bash
cargo run --bin multi-clients
```

Con esto se simulan múltiples usuarios que realizan operaciones concurrentes en el servidor gRPC mediante el uso de futures de Rust. Inicialmente con `MAX_USERS_TEST` especificado en [lib.rs](/src/lib.rs) se ejecutan 1024 clientes de prueba. Creándose de esta forma 1024 futures que se ejecutan en el runtime de tokio. A dichos futures se los ejecuta en un thread pool de 5 threads (por defecto) especificado también en la variable `MAX_T_SCHEDULING_USERS_TEST` en [lib.rs](/src/lib.rs). El runtime de tokio, para ejecutar cualquier tipo de futures del proceso, se encarga de procesarlas también en un pool de threads, los mismos se establecieron como máximo en 10 threads según lo especificado en la cfg del [main](/src/multi-clients.rs) del runtime de tokio.

Con este ejemplo de prueba se puede "jugar" y probar con las mencionadas variables y observar lo mencionado acerca de la ejecución concurrente de operaciones en el servidor gRPC. En el stdout de los clientes se puede observar la representación simulada de un cliente al estar invocando dicha future en distinto thread id del runtime de tokio. Si se scheduled en solo 1 thread, se observa como un comportamiento secuencial de los clientes. Distinto es si se scheduled en 10 threads, donde se observa una ejecución más rápida de los clientes por ejecutarse de forma concurrente con la programación asincrónica de Rust.

También en ese mismo ejemplo de prueba se puede modificar para ejecutar diversas operaciones de lectura y escritura tales como updates ,gets, creates o deletes.

### Servidor con un Cliente vía CLI

Teniendo el servidor ejecutado, se puede ejecutar un cliente mediante el siguiente comando:

```bash
cargo run --bin client <COMMAND>
```

Hay distintas opciones de COMMAND:
- get: Obtiene la información de un usuario específico según su ID (--id).
- get-all: Obtiene la información de todos los usuarios del sistema. Se puede limitar la cantidad de usuarios a obtener mediante el flag --limit.
- create: Crea un nuevo usuario con id, name y mail (--id, --name, --mail).
- delete: Elimina un usuario especificando según su ID (--id).
- update-name: Actualiza el nombre de un usuario especificando su ID y el nuevo name (--id, --name).
- update-mail: Actualiza el correo electrónico de un usuario, necesitará proporcionar su ID y el nuevo mail (--id, --mail).
- reset-table: Restablece la tabla de usuario, borrando todos los datos existentes.
- help: Proporciona una descripción detallada de todos los comandos disponibles.

Se puede obtener info de cada comando (para saber como pasarle los argumentos) mediante:

```bash
cargo run --bin client help <COMMAND>
```

Un ejemplo de uso simple:

```bash
cargo run --bin client get-all --limit 10
cargo run --bin client create --id 1 --name "Federico" --mail "fede@fede.ar"
cargo run --bin client get --id 1
cargo run --bin client update-name --id 1 --name "Pacheco"
cargo run --bin client get --id 1
cargo run --bin client delete --id 1
```

### Decisiones de Diseño y Reglas de Negocio

- No se puede crear un mismo usuario con un mismo id.
- El id, name y mail son obligatorios y se almacenan como string pero el mail debe ser válido.
- El id es único por usuario.
- Si se actualiza un campo con el mismo valor, retorna exito con el [status code](https://github.com/grpc/grpc/blob/master/doc/statuscodes.md#status-codes-and-their-use-in-grpc) OK del protocolo gRPC.
- La validacion de mails se podria haber evitado mediante uso de [Intercepts](https://docs.rs/tonic/latest/tonic/service/trait.Interceptor.html) en el servidor gRPC, pero se valida en cada endpoint. Idem como validación de ids, autenticación o cualquier otra validación de negocio o sistema.
- Utilizando las bondades de la programación asincrónica, se usan Futures en vez de Threads para la prueba de múltiples usuarios concurrentes debido a que son más livianos y eficientes que los threads. Ejecutar 1024 threads termina siendo muy costoso.
- Utilización de una base de datos "real" (usando docker) para los tests, se podría evitar con mocks.
  - En la ejecución de tests (debido a que Rust ejecuta tests en paralelo) se fixeó con un "contador" de tests para que se ejecute una sola vez la limpieza de la base de datos.
