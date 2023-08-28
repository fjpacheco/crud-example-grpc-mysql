# Documentación del Proyecto de Prueba para Candidatos en Kinsper

## Entorno de desarrollo 

Se desarrollo en el lenguaje de Rust, utilizando principalmente el crate de sqlx para el manejo de la base de datos MySQL, y el crate tonic para el mandejo del cliente y servidor gRPC. Para facilitar el desarrollo se utilizo los contenedores de Docker para la base de datos MySQL, de esta forma evitamos utilizar una base de datos remota.

## Acceso a la base de datos

Para la ejecucion y buildeo de la **base de datos de desarrollo** se ejecuta con docker, especificamente con docker-compose (por simplicidad de ejecucion de comandos, no habia necesidad de "orquestrar" N servicios con docker-compose):

```bash
docker-compose up --build
```

Y adicionalmente para parar la ejecucion de la base de datos se ejecuta:

```bash
docker-compose down
```

Para utilizar la **base de datos de produccion**, se debe solicitar el usuario, contraseña y url al desarrollador para asi ejecutar la base de datos MySQL provista por Google Cloud y actualizar el archivo [.env](.env) con las variables de entorno correspondientes.

## Ejecucion de entorno de pruebas 

Para ejecutar el entorno de pruebas, se necesita tener la base de datos local con docker ejecutandose (queda pendiente evitar esto usando mocks de la base de datos!) y ejecutar el siguiente comando:

```bash
cargo test
```

Ademas se necesita que la base de datos de desarrollo esté vacia para ciertos tests de 

## Acerca del manejo de errores

## Acerca del registro de actividad 

## Manual de Uso

### Servidor con Multiples Clientes 

### Servidor con un Cliente vía CLI

### Decisiones de Diseño

- No se puede crear un mismo usuario con un mismo id.
- El id, name y mail son obligatorios y se almacenan como string pero el mail debe ser valido.
- El id es unico por usuario.
- Si se actualiza un campo con el mismo valor, retorna exito con el [status code](https://github.com/grpc/grpc/blob/master/doc/statuscodes.md#status-codes-and-their-use-in-grpc) OK del protocolo gRPC.

