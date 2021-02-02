// DEPENDENCIAS
// extern crate postgres;
// use std::env;
use std::error::Error;
// use std::ffi::OsString;
// use std::fs::File;
use std::process;
use std::io;
use serde::Deserialize;
use regex::Regex;
use postgres::{Client, NoTls};

// ------------------------------------------------------------------------------------------------> DEFINICION DE PERSONA
#[derive(Debug, Deserialize)]
struct Persona {
    id: String,
    nombre: String,
    genero: String,
    civil: String,
    nacimiento: String,
    telefono: String,
    direccion: String,
    email: String,
}

// ------------------------------------------------------------------------------------------------> VALIDAR EL GENERO
fn gen_check(gender: &str) -> bool {

    if gender.to_uppercase() == "M" || gender.to_uppercase() == "F" || gender.to_uppercase() == "NULL" {
        // println!("Correcto!");
        return true;
    }
    else {
        println!("Genero no válido!");
        return false;
    }
}

// ------------------------------------------------------------------------------------------------> VALIDAR EL NOMBRE
fn name_check(name: &str) -> bool {
    let nw = name.split_whitespace().count(); // Number of Words
    let reg = Regex::new(r"[[:punct:][:digit:]]").unwrap();
    
    if nw < 2 {
        // println!("Nombre no válido. Debe contener al menos un nombre y un apellido.");
        return false; 
    }
    else if reg.is_match(name) {
        // println!("Nombre no válido. No puede contener numeros ni caracteres especiales");
        return false
    }
    else {
        return true;
    }   
}  

// ------------------------------------------------------------------------------------------------> PREPARANDO EL NOMBRE PARA LA INSERCION
fn name_prepare(name: &str) -> String {

    let accents = ["á","Á","é","É","í","Í","ó","Ó","ú","Ú"];
    let vowels= ["a","A","e","E","i","I","o","O","u","U"];

    let mut na = name.to_string();

    for i in 0..accents.len() {
        na = na.replace(accents[i],vowels[i]);
    }

    na = na.to_uppercase();
    return na;
}

// ------------------------------------------------------------------------------------------------> VALIDAR TELEFONO
fn tel_check(telnum: &str) -> String {
    let len = telnum.len();
    
    let num = Regex::new(r"[[:alpha:][:punct:]]").unwrap();
    let begins = Regex::new(r"^09").unwrap(); 
    
    if num.is_match(telnum) {
        return "Teléfono no válido. Debe contener unicamente numeros".to_string();
    }
    else if !begins.is_match(telnum) {
        return "Teléfono no válido. Debe comenzar con '09'.".to_string();
    }
    else if len < 10 {
        return "Teléfono no válido. Debe tener al menos 10 dígitos.".to_string();
    }
    else {
        return "Teléfono válido.".to_string();
    }
    
}

// ------------------------------------------------------------------------------------------------> LEER ARCHIVO CSV Y PROCESAMIENTO DE DATOS
fn run() -> Result<(), Box<dyn Error>> {
    // let mut rdr = csv::Reader::from_reader(io::stdin());
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b';')
        .double_quote(false)
        // .escape(Some(b'\\'))
        // .flexible(true)
        // .comment(Some(b'#'))
        .from_reader(io::stdin());

    let mut conex = Client::connect("postgresql://postgres:ibzeon@172.21.0.3:5432/kruger", NoTls)?;

    conex.batch_execute(
        "CREATE TABLE IF NOT EXISTS personas (
            id                  INTEGER NOT NULL,
            nombre              VARCHAR NOT NULL,
            genero              CHAR(1) NULL,
            estado_civil        VARCHAR NOT NULL,
            fecha_nacimiento    CHAR(10) NOT NULL,
            telefono            CHAR(16) NOT NULL,
            email               VARCHAR NOT NULL,
            valido              BOOLEAN NOT NULL,
            observacion         TEXT NULL
        )"
    )?;

    for result in rdr.deserialize() {
        let mut record: Persona = result?;
        // println!("{:?}", record);
        // println!("{}", record.nombre);
        let mut observacion = "";
        let mut valido = true;

        if gen_check(&record.genero) { // ---------------------------------------------------------> VALIDANDO EL GENERO
            record.genero = record.genero.to_uppercase();
            
            if !name_check(&record.nombre) { // ---------------------------------------------------> VALIDANDO EL NOMBRE
                observacion = "Nombre no válido";
                valido = false;
            }
            record.nombre = name_prepare(&record.nombre); // --------------------------------------> NOMBRE EN MAYUSCULAS SIN ACENTOS
            
            let tel = tel_check(&record.telefono);
            if tel != "Teléfono válido." { // -------------------------------> VALIDANDO TELEFONO
                observacion = &tel;
                valido = false;
            }
            

            println!("Agregando {:?} a la base de datos...", record);
            conex.execute(
                "INSERT INTO personas VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9)", 
                &[ &record.id.parse::<i32>().unwrap(), &record.nombre, &record.genero, &record.civil, &record.nacimiento, &record.telefono, &record.email, &valido, &observacion ]
            )?;





        }
        else {
            println!("La siguiente {:?} no se agregara a la base de datos", record);
        }

        println!("La insercion de datos ha terminado...");
    }
    
    Ok(())
}



fn main() {
    if let Err(err) = run() {
        println!("{}", err);
        process::exit(1);
    }
}
