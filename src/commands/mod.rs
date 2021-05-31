use crate::{messages::redis_messages, native_types::ErrorStruct};

pub mod database_mock;
pub mod sets;
pub mod strings;

// Duda 1:
// Usar el paquete MOD para algo así es posible? Son funciones que usarán todos los commandos
// he visto en creates externos que en los mod.rs dentro de una carpeta tienen tales funciones, y no se por qué
// tambien en los .lib externos..
// no se si son genericas a esa carpeta => @fjpacheco: yo le veo mucho sentido tirar acá en el mod.rs éste tipo de funciones que usan todos

// Duda 2: check_empty_and_name_command!(..)
// Que ventaja tiene hacer con macros esto? Será mas rapido? De Estruc. del Compu. entendí que con macros
// se expandirá el codigo en time compilacion => en criollo: se hace un copy-paste del codigo en donde la macro la haya invocado => claramente el codigo objeto generable será mas grande
// En cambio, las funciones es algo de tiempo de ejecucion, se realizará ese jump, luego retorno blabla, etc.
//
// En Rust... funciona igual?
// siento que hay microsegundos que se pierden al hacer este chequeo de empty/name con una funcion tan repetitiva
// en casi TODO los comandos a implementar. Y usar macros, no vendría mal
// podria usarse para esa forma las macros de Rust? mmmmmmmmmmmm

// Duda 3: esto podria no chequearse y estar en un nivel mas arriba de jerarquia de chequeos de comandos en general.. ojo!

// TODO: consultar Rust-eze team/Matías

fn check_empty_and_name_command(buffer: &&mut Vec<&str>, name: &str) -> Result<(), ErrorStruct> {
    if buffer.is_empty() {
        let message_error = redis_messages::not_empty_values_for(name);
        return Err(ErrorStruct::new(
            message_error.get_prefix(),
            message_error.get_message(),
        ));
    }
    let command_lowercase = buffer[0].to_lowercase();
    if !command_lowercase.eq(name) {
        let concat_vector_buffer = buffer.join(" ");
        let error_message = redis_messages::command_not_found_in(concat_vector_buffer);
        return Err(ErrorStruct::new(
            error_message.get_prefix(),
            error_message.get_message(),
        ));
    }
    Ok(())
}

#[macro_export]
macro_rules! wrongtype {
    () => {
        Err(ErrorStruct::new(
            redis_messages::wrongtype().get_prefix(),
            redis_messages::wrongtype().get_message(),
        ))
    };
}
