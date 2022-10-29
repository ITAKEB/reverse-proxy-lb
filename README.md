# Proxy Inverso y Balanceador de carga en Rust

## Contribuidores

- David Jose Cardona Nieves - Alias: El convierte lesbianas
- Tomas Atehortua Ceferino
- Danilo Toro Echeverri

## Introducción

El programa desarrollado es un proxy inverso con balanceador de carga en el lenguaje de programación Rust. Un proxy inverso es un servidor que se sitúa delante de uno o varios servidores web, interceptando las peticiones de los clientes y enviando las peticiones a los servidores web con el objetivo de obtener una respuesta para enviarla al cliente. Un proxy inversos suele implementarse para ayudar a aumentar la seguridad y el rendimiento [1]. Además, para implementar el balanceador de carga se ha utilizado la regla Round Robin, con el objetivo de distribuir la carga entre los diferentes servidores web [2].

Los servidores han sido desplegados en instancias EC2 de Amazon Web Services.

 - La versión de Http soportada es la 1.1.
 - No hay soporte para GZIP, codificación ideal para el envío de archivos pesados. No importa que el cliente añada la opción, el proxy la elimina.

## Desarrollo

### Concurrencia

El sistema debe ser capaz de responder a varias peticiones simultáneas, por lo cual se ha trabajado con `threads` para evitar que el proxy se bloquee mientras está procesando una petición, aumentando el rendimiento.

Para manipular varios threads se usó un `thread pool` [3]. Un thread pool es un grupo de hilos generados que están a la espera de ser asignados. Cuando el programa recibe una nueva petición, este asigna a alguno de los threads disponibles en el momento. Cuando el thread termina de procesar su tarea, es devuelto al thread pool de threads disponibles para ser asignados a otra petición.

Implementación `ThreadPool`
```
pub struct ThreadPool {
	workers: Vec<Worker>,
	sender: mpsc::Sender<Job>,
}
```
Donde `workers` es un arreglo de threads (los que serán usados para responder a cada petición) y `sender` es el encargado de compartir la función a ejecutar `Job`.  Por consecuencia, cada vez que llegue una petición se tomara un threads disponible y se le asignara una petición con el método `execute` en `src/threadpool`. Debido a que  los threads en Rust se comienzan a ejecutar una vez sean creados se debe desarrollar un método para asignarle un trabajo a ejecutar en cada momento [4].
```
pub fn execute<F>(&self, f: F)
where
F: FnOnce() + Send + 'static,
{
	let job = Box::new(f);
	if let Err(_) = self.sender.send(job) {
		println!("Error: Did not send job");
	}
}
```
Se debe decidir la cantidad de threads a crear, debido a que no se pueden crear threads según la cantidad de peticiones recibidas porque las peticiones pueden ser muy masivas y crear una gran cantidad de hilos puede disminuir el rendimiento de nuestro proxy. Para solventar ese problema, se asignará un número fijo de hilos esperando en el pool asignados en el archivo de configuración `src/config.rs`. Y no existirá el problema de perder peticiones en el camino ya que el pool mantendrá una cola de peticiones entrantes. Cada uno de los hilos en el pool solicitará una solicitud y una vez procesada sigue solicitando. Con este diseño, podemos procesar hasta N peticiones simultáneamente, donde N es el número de hilos.

### Lectura petición Http y cache

Con la utilidad TCPListener `TcpListener::bind`, permite que el proxy se encuentre a la escucha en algún puerto disponible. Con la idea de generar una conexión TCPStream con el cliente una vez llegue una petición. De este modo, podemos leer las peticiones y enviar las respuestas.

Al leer la petición del cliente mediante el `Buffer`, lo siguiente a ejecutar es la verificación del tipo. Ya que si es un `GET`, puede haber sido almacenado previamente por el `caché` o ser susceptible a almacenar. También, puede ser un `POST`, por lo que se verifica si tiene `content-length`ya que puede contener `body` para ser leído.

La idea principal de cache es la siguiente:

Si la petición es susceptible a hacer caché se ha creado un `hilo independiente` el cual tratará de guardar la información en el directorio de la máquina. Se hace en un hilo diferente del hilo principal que recibe la petición debido a que es una operación diferente y no debería de ser retrasada la respuesta por la escritura del caché.

En el archivo que se almacenará la información del `body`se le añade información extra para facilitar su manipulación. Como el `tiempo que fue creado` y `el tiempo de vida del archivo` (por defecto todo archivo tiene el mismo TTL). Una vez se haya creado el archivo con la información necesaria se crea el sistema de directorios. El sistema depende de la ruta que la misma petición contiene en el `status line`.

Si el sistema encuentra la ruta del archivo solicitado en la petición, se creará una respuesta con el contenido del archivo para ser enviada al cliente. Cada vez que respondamos de esta forma se analiza si el archivo aún debe seguir almacenado o debe ser eliminado. Si es el caso de que deba ser eliminado, se usa el mismo hilo que escribe el archivo en caché, pero en este caso eliminará el archivo correspondiente.

### Envío de petición al servidor web y respuesta al cliente

Si la petición no se encuentra en el caché o no es susceptible (es un método diferente a GET), se deberá hacer la petición al servidor web. Antes de realizar la petición se `limpia la petición con funcionalidades que no soportamos`. Además, cambiar el `host del cliente` en la petición por el del proxy.

Una vez recibida la respuesta del servidor web, se la enviamos al cliente. Si en algún momento se produce un fallo, el proxy se ve obligado a responder al cliente con alguna de las posibles respuestas a errores de Http, siempre y cuando la conexión con el cliente siga activa.

## Ejecución

Para ejecutar el proxy se debe hacer lo siguiente:

Ofrecer permisos al script bash con el comando:
```
chmod +X ./first-install-and-run.sh
```

El script lo que va a ser es:

 - Actualizar el sistema de enlaces de Ubuntu.
 - Instalar `curl`.
 - Instalar el compilador de Rust.
 - Ejecutar el programa.

## Conclusiones

El proxy es capaz de responder a algunas peticiones simultáneas sin bloquearse. Por consecuencia, el proxy es capaz de defenderse de algunos ataques de DoS. Además de no ignorar peticiones sin importar que en el momento no puedan ser procesadas..

Para un futuro desarrollo se desea mejorar la forma en que el balanceador de carga es usado. Es decir, implementar más políticas para mejorar la asignación, como conocer el estado actual de los servidores web y de esta forma asignar más tareas a servidores web más rápidos y se encuentren con menos peticiones en curso.

## Referencias
- [1] https://www.nginx.com/resources/glossary/reverse-proxy-server/
- [2] https://avinetworks.com/glossary/round-robin-load-balancing/
- [3] https://applied-math-coding.medium.com/implementing-a-basic-thread-pool-in-rust-cd8a00363942
- [4] https://doc.rust-lang.org/book/ch20-02-multithreaded.html
