# CRUD con gRPC y MySQL en Rust 

Prueba tecnica donde se desarrolla una aplicacion sencilla en Rust que permite a los usuarios interactuar con una base de datos MySQL, utilizando gRPC para la comunicación.

## Entorno de desarrollo 

Se desarrollo en el lenguaje de Rust, utilizando principalmente el crate de [sqlx](https://crates.io/crates/sqlx) para el manejo de la base de datos MySQL, y el crate [tonic](https://crates.io/crates/tonic) para el mandejo del cliente y servidor gRPC. Para facilitar el desarrollo se utilizo los contenedores de Docker para la base de datos MySQL evitando el uso de una base de datos remota.

## Acceso a la base de datos

Para la ejecucion y buildeo de la **base de datos de desarrollo** mediante docker, especificamente con docker-compose (por simplicidad de ejecucion de comandos, no habia necesidad de "orquestrar" N servicios con docker-compose):

```bash
docker-compose up --build
```

Y adicionalmente para parar la ejecucion de la base de datos se ejecuta:

```bash
docker-compose down
```

Para utilizar la **base de datos de produccion**, se debe solicitar la contraseña y host al desarrollador para actualizar el archivo [.env](.env) con las variables de entorno correspondientes. De esta forma se podra ejecutar la base de datos MySQL provista por Google Cloud. 

## Ejecucion de entorno de pruebas 

Para ejecutar el entorno de pruebas, se necesita tener la base de datos local con docker ejecutandose (¡queda pendiente resolverlo con [mocks](https://asomers.github.io/mock_shootout/) en Rust para evitar usar una conexion real a una base de datos en los tests!) y ejecutar el siguiente comando mediante cargo:

```bash
cargo test
```
O mediante el archivo makefile donde ya levanta la MySQL, ejecuta los tests y apaga la MySQL:

```bash
make test
```
Precaución: se necesita que la base de datos de desarrollo esté vacia por si hay repetición de ids en los tests con ids previos existentes en la tabla de usuarios (¡esto se resolveria con mocks en Rust!), el cliente via CLI puede ayudar a resetear la tabla de usuarios mediante el comando `reset-table`.

Queda pendiente testear los endpoints del servidor/client de gRPC con el crate tonic de Rust.

Adicionalmente en cada push a main se ejecuta un pipeline de Github Actions que ejecuta los tests (que a su vez levanta una base de datos de desarrollo con Docker Hub de GitHub Actions), como tambien ejecuta los linters (format & clippy) y el build. 

## Acerca del registro de actividad 

Se utilizo el crate [log](https://crates.io/crates/log) para el manejo de logs del sistema. Actualmente todo tipo de log se muestra en el _stdout_ del servidor y se visualizan las operaciones Info del acceso a la base de datos como el tipo de operacion que realiza algun cliente. El nivel inicial de log es de Info, pero se puede modificar en el [lib.rs](src/lib.rs) o sino ejecutando el servidor con la variable de entorno RUST_LOG=debug. 

Con RUST_LOG=debug se podra observar en qué thread id del runtime de tokio se ejecuta la peticion al servidor gRPC. Esto es util para ver que en el servidor gRPC se ejecutan operaciones de forma concurrente (y al usar una base de datos relacional, nos garantiza ACID para estas operaciones concurrentes). Este efecto se puede observar en mayor detalle cuando se ejecuta el servidor con multiples clientes simulados.

## Acerca del manejo de errores

Los errores se manejan mediante el uso de Results en Rust, como el operador ? para propagar errores. Los errores de la base de datos se manejan en el archivo [db.rs](src/errors.rs) el cual se encarga de convertir (mediante el trait From) cada error del crate sqlx a un error del negocio (ErrorKinsper). A su vez cada error del negocio, en ese mismo archivo se convierte a un error de gRPC (Status) para ser enviado al cliente. De esta forma ganamos un manejo de errores mas robusto y mantenible, ubicando el manejo de errores en un solo lugar mediante el uso de las caracteristicas de Rust.

## Manual de Uso

### Ejecucion del Servidor

Para la ejecucion del servidor mediante cargo:

```bash
cargo run --bin server
```

Previamante hay que tener en [.env](.env) las variables de entorno correspondientes para la conexion a la base de datos (sea asi de desarrollo o produccion).

### Servidor con Multiples Clientes 

Teniendo el servidor ejecutado, se puede ejecutar multiples clientes mediante el siguiente comando:

```bash
cargo run --bin multi-clients
```

Con esto se simulan multiples usuarios que realizan operaciones concurrentes en el servidor gRPC mediante el uso de futures de Rust. Inicialmente con `MAX_USERS_TEST` especificado en [lib.rs](/src/lib.rs) se ejecutan 1024 clientes de prueba. Creandose de esta forma 1024 futures que se ejecutan en el runtime de tokio. A dichos futures se los ejecuta en un thread pool de 5 threads (por defecto) especificado tambien en la variable `MAX_T_SCHEDULING_USERS_TEST` en [lib.rs](/src/lib.rs). En el runtime de tokio se ejecutan multiples threads que se pueden ejecutar hasta 10 threads segun lo especificado en la cfg del [main](/src/multi-clients.rs) del runtime de tokio.

Con este ejemplo de prueba se puede "jugar" y probar con las mencioandas variables y observar lo mencionado acerca de la ejecucion concurrente de operaciones en el servidor gRPC. En el stdout de los clientes se puede observar la representacion simulada de un cliente al estar invocandose dicha future en distinto thread id del runtime de tokio. Si se schedulea en solo 1 thread, se observara como un comportamiento secuencial de los clientes. Distinto es si se schedulea en 10 threads, donde se observara una ejecucion mas rapida de los clientes por ejecutarse de forma concurrente con la programacion asincronica de Rust.

Tambien en ese mismo ejemplo de prueba se puede modificar para ejecutar diversas operaciones de lectura y escritura tales como updates ,gets, creates o deletes.

### Servidor con un Cliente vía CLI

Teniendo el servidor ejecutado, se puede ejecutar un cliente mediante el siguiente comando:

```bash
cargo run --bin client <COMMAND>
```

Hay distintas opciones de COMMAND:
- get: Obtiene la informacion de un usuario especifico segun su ID (--id).
- get-all: Obtiene la informacion de todos los usuarios del sistema. Se puede limitar la cantidad de usuarios a obtener mediante el flag --limit.
- create: Crea un nuevo usuario con id, name y mail (--id, --name, --mail).
- delete: Elimina un usuario especificando segun su ID (--id).
- update-name: Actualiza el nombre de un usuario especificando su ID y el nuevo name (--id, --name).
- update-mail: Actualiza el correo electrónico de un usuario, necesitas proporcionar su ID y el nuevo mail (--id, --mail).
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
- El id, name y mail son obligatorios y se almacenan como string pero el mail debe ser valido.
- El id es unico por usuario.
- Si se actualiza un campo con el mismo valor, retorna exito con el [status code](https://github.com/grpc/grpc/blob/master/doc/statuscodes.md#status-codes-and-their-use-in-grpc) OK del protocolo gRPC.
- La validacion de mails se podria haber evitado mediante uso de [Intercepts](https://docs.rs/tonic/latest/tonic/service/trait.Interceptor.html) en el servidor gRPC, pero se valida en cada endpoint. Idem como validacion de ids, autenticacion o cualquier otra validacion de negocio o sistema.
- Utilizando las bondades de la programacion asincronica, se usan Futures en vez de Threads para la prueba de multiples usuarios concurrentes debido a que son mas livianos y eficientes que los threads. Ejecutar 1024 threads termina siendo muy costoso.
  