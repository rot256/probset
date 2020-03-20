use log::info;

use regex::Regex;

use yew::{html, Component, ComponentLink, Html, ShouldRender};

use std::option::NoneError;

use super::filters::cuckoo;

/// Specified parameters for all filters
#[derive(Debug)]
pub struct Params {
    error: String,
    elements: String,
    storage: String,
}

/// Cuckoo filter hyper parameters
pub struct CuckooParams {}

pub struct Model {
    params: Params,
    link: ComponentLink<Self>,
}

impl Model {
    pub fn initialize(&mut self) {}
}

pub enum Msg {
    UpdateError(String),
    UpdateElements(String),
    UpdateStorage(String),
}

fn is_space(s: &str) -> bool {
    s.trim().len() == 0
}

fn parse_error(s: &str) -> Result<f64, NoneError> {
    let re = Regex::new(r"^[ ]*([\d\.]+)[ ]*(%)?[ ]*$").unwrap();
    let caps = re.captures(s)?;

    // parse "base" number
    let f = caps.get(1)?.as_str().parse::<f64>().ok()?;

    let f = match caps.get(2) {
        Some(_) => f / 100.,
        None => f,
    };

    if f <= 0. || f >= 1. {
        Err(NoneError)
    } else {
        Ok(f)
    }
}

fn parse_elements(s: &str) -> Result<u64, NoneError> {
    let re = Regex::new(r"^[ ]*(\d+)[ ]*([KMGT])?[ ]*$").unwrap();
    let caps = re.captures(s)?;

    // parse "base" number
    let num = caps.get(1)?.as_str().parse::<u64>().ok()?;

    // parse multiplier
    let num = match caps.get(2).map(|s| s.as_str()) {
        None => Some(num),
        Some("K") => num.checked_mul(1_000),
        Some("M") => num.checked_mul(1_000_000),
        Some("G") => num.checked_mul(1_000_000_000),
        Some("T") => num.checked_mul(1_000_000_000_000),
        _ => unreachable!("regex should not match"),
    }?;

    Ok(num)
}

fn parse_storage(s: &str) -> Result<u64, NoneError> {
    let re =
        Regex::new(r"^[ ]*(\d+)[ ]*((KB)|(K)|(Kb)|(KiB)|(MB)|(M)|(Mb)|(MiB)|(GB)|G|(Gb)|(GiB)|(TB)|(T)|(Tb)|(TiB))?[ ]*$").unwrap();
    let caps = re.captures(s)?;

    // parse "base" number
    let num = caps.get(1)?.as_str().parse::<u64>().ok()?;

    // parse multiplier
    let num = match caps.get(2).map(|s| s.as_str()) {
        None => Some(num),
        // SI bits
        Some("K") | Some("Kb") => num.checked_mul(1_000),
        Some("M") | Some("Mb") => num.checked_mul(1_000_000),
        Some("G") | Some("Gb") => num.checked_mul(1_000_000_000),
        Some("T") | Some("Tb") => num.checked_mul(1_000_000_000_000),
        // SI bytes
        Some("KB") => num.checked_mul(8 * 1_000),
        Some("MB") => num.checked_mul(8 * 1_000_000),
        Some("GB") => num.checked_mul(8 * 1_000_000_000),
        Some("TB") => num.checked_mul(8 * 1_000_000_000_000),
        // 2^10 multiples
        Some("KiB") => num.checked_mul(8 * 1024),
        Some("MiB") => num.checked_mul(8 * 1024 * 1024),
        Some("GiB") => num.checked_mul(8 * 1024 * 1024 * 1024),
        Some("TiB") => num.checked_mul(8 * 1024 * 1024 * 1024 * 1024),
        _ => unreachable!("regex should not match"),
    }?;

    Ok(num)
}

fn count_some<T, E>(v: Result<Option<T>, E>) -> u32 {
    if let Ok(v) = v {
        v.is_some() as u32
    } else {
        0
    }
}

impl Model {
    fn render_cuckoo(
        &self,
        storage: Result<Option<u64>, NoneError>,
        elements: Result<Option<u64>, NoneError>,
        error: Result<Option<f64>, NoneError>,
    ) -> Html {
        let err = storage.is_err() | elements.is_err() | error.is_err();

        let error = if err { None } else { error.unwrap() };
        let storage = if err { None } else { storage.unwrap() };
        let elements = if err { None } else { elements.unwrap() };

        let params = cuckoo::Parameters::new(error, elements, storage, 2, 4, 0.95);

        html! {
            <table class="mono">
                <tr>
                    <td>{"Fingerprint size"}</td>
                    <td>{":"}</td>
                    <td>{ if let Some(fingerprint) = params.fingerprint() {
                        format!("{} bits", fingerprint)
                    } else {
                        "".to_string()
                    } }</td>
                </tr>
                <tr>
                    <td>{"Number of items in filter"}</td>
                    <td>{":"}</td>
                    <td>{ if let Some(elements) = params.elements() {
                        format!("{}", elements)
                    } else {
                        "".to_string()
                    } }</td>
                </tr>
                <tr>
                    <td>{"False positive rate"}</td>
                    <td>{":"}</td>
                    <td>{ if let Some(error) = params.error() {
                        format!("{} (1 in {})", error, f64::round(1. / error))
                    } else {
                        "".to_string()
                    } }</td>
                </tr>
                <tr>
                    <td>{"Bits per item"}</td>
                    <td>{":"}</td>
                    <td>{ if let Some(bits_per_element) = params.bits_per_element() {
                        format!("{} bits/item", bits_per_element)
                    } else {
                        "".to_string()
                    } }</td>
                </tr>
            </table>
        }
    }

    fn render_input(
        &self,
        storage: Result<Option<u64>, NoneError>,
        elements: Result<Option<u64>, NoneError>,
        error: Result<Option<f64>, NoneError>,
    ) -> Html {
        let cons = count_some(storage) + count_some(elements) + count_some(error);
        let too_many = cons > 2;
        html! {
            <div>
            <form>
                <h2>{"Constraints:"}</h2>
                <fieldset>
                    <legend>{"Size of the filter (bits):"}</legend>
                    <input
                        placeholder="Size of the filter in"
                        type="text"
                        id="input-storage"
                        class={
                            if storage.is_err() || too_many {
                                "parse-error"
                            } else {
                                "parse-ok"
                            }
                        }
                        value={&self.params.storage}
                        oninput=self.link.callback(move |e: html::InputData| {
                            Msg::UpdateStorage(e.value)
                        })
                    />
                    <p class="sub-text">
                        {
                            if let Ok(Some(storage)) = storage {
                                format!("{} Bits", storage)
                            } else {
                                "Supports units, e.g. K, KiB, KB, etc.".to_string()
                            }
                        }
                    </p>
                </fieldset>
                <span></span>
                <fieldset>
                    <legend>{"Number of items in the filter:"}</legend>
                    <input
                        placeholder="Total items in filter"
                        type="text"
                        id="input-elements"
                        class={
                            if elements.is_err() || too_many {
                                "parse-error"
                            } else {
                                "parse-ok"
                            }
                        }
                        value={&self.params.elements}
                        oninput=self.link.callback(move |e: html::InputData| {
                            Msg::UpdateElements(e.value)
                        })
                    />
                    <p class="sub-text">
                        {
                            if let Ok(Some(elements)) = elements {
                                format!("{} Items", elements)
                            } else {
                                "Supports SI units: K, G, M, T".to_string()
                            }
                        }
                    </p>
                </fieldset>
                <fieldset>
                    <legend>{"False positive rate:"}</legend>
                    <input
                        placeholder="False positive rate"
                        type="text"
                        id="input-error"
                        class={
                            if error.is_err() || too_many  {
                                "parse-error"
                            } else {
                                "parse-ok"
                            }
                        }
                        value={&self.params.error}
                        oninput=self.link.callback(move |e: html::InputData| {
                            Msg::UpdateError(e.value)
                        })
                    />
                </fieldset>
            </form>
            </div>
        }
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Model {
            link,
            params: Params {
                error: "0.01".to_string(),
                elements: "1G".to_string(),
                storage: "".to_string(),
            },
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UpdateElements(s) => {
                self.params.elements = s;
            }
            Msg::UpdateError(s) => {
                self.params.error = s;
            }
            Msg::UpdateStorage(s) => {
                self.params.storage = s;
            }
        }
        true
    }

    fn view(&self) -> Html {
        log::info!("parameters: {:?}", self.params);
        info!("update view");

        // parse storage string (or unspecified)
        let storage = if is_space(&self.params.storage) {
            Ok(None)
        } else {
            parse_storage(&self.params.storage).map(|v| Some(v))
        };

        // parse elements string (or unspecified)
        let elements = if is_space(&self.params.elements) {
            Ok(None)
        } else {
            parse_elements(&self.params.elements).map(|v| Some(v))
        };

        // parse false positive rate
        let error = if is_space(&self.params.error) {
            Ok(None)
        } else {
            parse_error(&self.params.error).map(|v| Some(v))
        };

        html! {
            <div>
                {self.render_input(storage, elements, error)}
                <div>
                    <h2>{"Cuckoo Filter"}</h2>
                    {self.render_cuckoo(storage, elements, error)}
                </div>
                <div>
                    <h2>{"Bloom Filter"}</h2>
                </div>
                <footer>
                    <hr></hr>
                    <center>
                    <p>{"Mathias Hall-Andersen"}</p>
                    </center>
                </footer>
            </div>
        }
    }
}
