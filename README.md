# Proyecto de Prueba para Candidatos en Kinsper

## Objetivo

Desarrollar una aplicación sencilla en Rust que permita a los usuarios interactuar con una base de datos MySQL, utilizando gRPC para la comunicación.

## Características Técnicas Requeridas:

### 1. Conexión y Esquema en MySQL:

- Establecer una conexión con una base de datos MySQL.
- Diseñar un esquema sencillo que incluya una tabla usuarios con campos id, nombre y correo.
- Implementar operaciones CRUD básicas para gestionar la información de los usuarios.
  
### 2. Comunicación y Concurrency con gRPC:

- Configurar un servidor gRPC en Rust que ofrezca endpoints para las operaciones CRUD de los usuarios.
- Desarrollar un cliente gRPC que interactúe con el servidor. Debe incluir una función en la que varios usuarios simulados (representados como hilos) intenten leer y escribir en la base de datos simultáneamente, mostrando el manejo de concurrencia.

### 3. Características no técnicas:

- Interfaz: Proveer una interfaz básica (CLI o GUI) para que el usuario pueda interactuar con las funciones de la aplicación.
- Documentación: Incluir una guía sobre cómo usar y probar la aplicación, junto con la justificación de las decisiones técnicas tomadas.

### Puntos Extra (Opcional):

- Manejo de errores: Implementar una gestión básica de errores, especialmente para posibles fallos en la conexión con la base de datos.
- Registro de Actividad: Implementar un mecanismo sencillo para registrar las operaciones realizadas en el servidor, ya sea un log o una bitácora.
  
### Entrega:

- Repositorio en GitHub con el código fuente.
- Acceso o demostración en un entorno de prueba.
- Acceso a la base de datos para validar los datos.

### Contexto y Consideraciones:

Buscamos un desarrollador para integrarse a nuestro equipo en un proyecto mayor. Este ejercicio tiene como fin evaluar tus habilidades y calidad de trabajo. Sólo aquellos candidatos dispuestos a colaborar en un proyecto más grande deben postularse.
 