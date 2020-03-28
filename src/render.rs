use log::info;

use regex::Regex;

use yew::{html, Component, ComponentLink, Html, ShouldRender};

use std::option::NoneError;

use super::filters::{bloom, cuckoo, theory, FilterParameters};

/// Specified parameters for all filters
#[derive(Debug)]
pub struct Params {
    error: String,
    elements: String,
    storage: String,
    cuckoo_hashes: u64,
    cuckoo_slots: u64,
}

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
    UpdateCuckooHashes(u64),
    UpdateCuckooSlots(u64),
}

fn is_space(s: &str) -> bool {
    s.trim().len() == 0
}

fn parse_error(s: &str) -> Result<f64, NoneError> {
    let re = Regex::new(r"^[ ]*(.+?)[ ]*(%)?[ ]*$").unwrap();
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

// TODO: make it not horrible
fn sep_1000(v: u64) -> String {
    let mut s = v.to_string();
    let mut r = String::new();
    loop {
        if s.len() <= 3 {
            break if r.is_empty() {
                s
            } else {
                format!("{},{}", s, r)
            };
        } else {
            let (pre, post) = s.split_at(s.len() - 3);
            if r.is_empty() {
                r = post.to_string();
            } else {
                r = format!("{},{}", post, r);
            }
            s = pre.to_string();
        }
    }
}

fn count_some<T, E>(v: Result<Option<T>, E>) -> u32 {
    if let Ok(v) = v {
        v.is_some() as u32
    } else {
        0
    }
}

fn format_error(error: f64) -> String {
    let parts: f64 = 1. / error;

    if parts.is_finite() {
        format!("{} (2^{:.2})", error, f64::log2(error))
    } else {
        format!("negligible (< 2^-50)")
    }
}

fn render_param_elements<F: FilterParameters>(params: &F) -> Html {
    html! {
        <tr>
            <td>{"Number of items in filter"}</td>
            <td>{":"}</td>
            <td>{ if let Some(elements) = params.elements() {
                format!("{}", elements)
            } else {
                "".to_string()
            } }</td>
        </tr>
    }
}
fn render_param_error<F: FilterParameters>(params: &F) -> Html {
    html! {
        <tr>
            <td>{"False positive rate"}</td>
            <td>{":"}</td>
            <td>{ if let Some(error) = params.error() {
                format_error(error)
            } else {
                "".to_string()
            } }</td>
        </tr>
    }
}

fn render_param_storage<F: FilterParameters>(params: &F) -> Html {
    html! {
        <tr>
            <td>{"Storage"}</td>
            <td>{":"}</td>
            <td>{ if let Some(storage) = params.storage() {
                let (div, name) = match storage {
                    x if x > 8 * 1024 * 1024 * 1024 * 1024 => (8 * 1024 * 1024 * 1024 * 1024, "PiB"),
                    x if x > 8 * 1024 * 1024 * 1024 => (8 * 1024 * 1024 * 1024, "TiB"),
                    x if x > 8 * 1024 * 1024 => (8 * 1024 * 1024, "GiB"),
                    x if x > 8 * 1024 => (8 * 1024, "KiB"),
                    _ => (1, "B")
                };
                if let Some(n) = storage.checked_mul(100) {
                    if n == 0 {
                        "overflow".to_string()
                    } else {
                        let n = n / div;
                        format!("{} bits ({}.{} {})", sep_1000(storage), n / 100, n % 100, name)
                    }
                } else {
                    "overflow".to_string()
                }
            } else {
                "".to_string()
            } }</td>
        </tr>
    }
}

fn render_param_bits<F: FilterParameters>(params: &F) -> Html {
    html! {
        <tr>
            <td>{"Bits per item"}</td>
            <td>{":"}</td>
            <td>{ if let Some(bits_per_element) = params.bits_per_element() {
                format!("{:.2} bits/item", bits_per_element)
            } else {
                "".to_string()
            } }</td>
        </tr>
    }
}

impl Model {
    fn render_theory(
        &self,
        storage: Result<Option<u64>, NoneError>,
        elements: Result<Option<u64>, NoneError>,
        error: Result<Option<f64>, NoneError>,
    ) -> Html {
        let err = storage.is_err() | elements.is_err() | error.is_err();

        let error = if err { None } else { error.unwrap() };
        let storage = if err { None } else { storage.unwrap() };
        let elements = if err { None } else { elements.unwrap() };

        let params = theory::Parameters::new(error, elements, storage);

        html! {
            <table class="mono">
                { render_param_storage(&params) }
                { render_param_elements(&params) }
                { render_param_error(&params) }
                { render_param_bits(&params) }
            </table>
        }
    }

    fn render_bloom(
        &self,
        storage: Result<Option<u64>, NoneError>,
        elements: Result<Option<u64>, NoneError>,
        error: Result<Option<f64>, NoneError>,
    ) -> Html {
        let err = storage.is_err() | elements.is_err() | error.is_err();

        let error = if err { None } else { error.unwrap() };
        let storage = if err { None } else { storage.unwrap() };
        let elements = if err { None } else { elements.unwrap() };

        let params = bloom::Parameters::new(error, elements, storage, None);

        html! {
            <table class="mono">
                { render_param_storage(&params) }
                { render_param_elements(&params) }
                { render_param_error(&params) }
                { render_param_bits(&params) }
                <tr></tr>
                <tr class="specific">
                    <td>{"Hashes"}</td>
                    <td>{":"}</td>
                    <td>{ if let Some(hashes) = params.hashes() {
                        format!("{}", hashes)
                    } else {
                        "".to_string()
                    } }</td>
                </tr>
            </table>
        }
    }

    fn render_cuckoo(
        &self,
        storage: Result<Option<u64>, NoneError>,
        elements: Result<Option<u64>, NoneError>,
        error: Result<Option<f64>, NoneError>,
        hashes: u64,
        slots: u64,
    ) -> Html {
        let err = storage.is_err() | elements.is_err() | error.is_err();

        let error = if err { None } else { error.unwrap() };
        let storage = if err { None } else { storage.unwrap() };
        let elements = if err { None } else { elements.unwrap() };

        let params = cuckoo::Parameters::new(error, elements, storage, hashes, slots, 0.95, true);

        html! {
            <table class="mono">
                { render_param_storage(&params) }
                { render_param_elements(&params) }
                { render_param_error(&params) }
                { render_param_bits(&params) }
                <tr class="specific">
                    <td>{"Fingerprint size"}</td>
                    <td>{":"}</td>
                    <td>{ if let Some(fingerprint) = params.fingerprint() {
                        format!("{} bits", fingerprint)
                    } else {
                        "".to_string()
                    } }</td>
                </tr>
                <tr class="specific">
                    <td>{"Buckets"}</td>
                    <td>{":"}</td>
                    <td>{ if let Some(buckets) = params.buckets() {
                        buckets.to_string()
                    } else {
                        "".to_string()
                    } }</td>
                </tr>
                <tr class="specific">
                    <td>{"Slots per bucket"}</td>
                    <td>{":"}</td>
                    <td>{ params.slots() }</td>
                </tr>
                <tr class="specific">
                    <td>{"Hashes"}</td>
                    <td>{":"}</td>
                    <td>{ params.hashes() }</td>
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
                <p>{"You must specify exactly two of these constraints (the third is inferred):"}</p>
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
                    <legend>{"Maximum false positive rate:"}</legend>
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
                error: "0.0000001".to_string(),
                elements: "4K".to_string(),
                storage: "".to_string(),
                cuckoo_hashes: 2,
                cuckoo_slots: 4,
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
            Msg::UpdateCuckooHashes(n) => {
                self.params.cuckoo_hashes = n;
            }
            Msg::UpdateCuckooSlots(n) => {
                self.params.cuckoo_slots = n;
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
                    <h4>{"Theoretic Limit"}</h4>
                    {self.render_theory(storage, elements, error)}
                </div>
                <div>
                    <h4>{"Cuckoo Filter"}</h4>
                    {self.render_cuckoo(storage, elements, error, self.params.cuckoo_hashes, self.params.cuckoo_slots)}
                    /*
                    <br></br>
                    <fieldset>
                        <legend>{"Cuckoo Filter Hyperparameters:"}</legend>
                        <table>
                            <tr>
                                <td>{"Hashes"}</td>
                                <td>{ ":" }</td>
                                <td style="width: 2em">{ self.params.cuckoo_hashes }</td>
                                <td>
                                    <input type="range" min="2" max="32" value="2" class="slider" oninput=self.link.callback(move |e: html::InputData| {
                                        Msg::UpdateCuckooHashes(e.value.parse().unwrap())
                                    })></input>
                                </td>
                            </tr>
                            <tr>
                                <td>{"Slots per bucket"}</td>
                                <td>{ ":" }</td>
                                <td style="width: 2em">{ self.params.cuckoo_slots }</td>
                                <td>
                                    <input type="range" min="4" max="32" value="4" class="slider" oninput=self.link.callback(move |e: html::InputData| {
                                        Msg::UpdateCuckooSlots(e.value.parse().unwrap())
                                    })></input>
                                </td>
                            </tr>
                        </table>
                    </fieldset>
                    */
                </div>
                <div>
                    <h4>{"Bloom Filter"}</h4>
                    {self.render_bloom(storage, elements, error)}
                </div>
                <h4>{"Resources"}</h4>
                <ul>
                    <li><a href="https://en.wikipedia.org/wiki/Bloom_filter">{"Bloom filter (wikipedia)"}</a></li>
                    <li><a href="https://en.wikipedia.org/wiki/Cuckoo_filter">{"Cuckoo filter (wikipedia)"}</a></li>
                    <li><a href="https://www.cs.cmu.edu/~dga/papers/cuckoo-conext2014.pdf">{"Cuckoo Filter: Practically Better Than Bloom"}</a></li>
                    <li><a href="https://www.vldb.org/pvldb/vol11/p1041-breslow.pdf">{"Morton Filters: Faster, Space-Efficient Cuckoo Filters via Biasing, Compression, and Decoupled Logical Sparsity"}</a></li>
                </ul>
                <footer>
                    <hr></hr>
                    <center>
                    <p>{ "Provided without warranty. I take no responsibility for the accuracy of the calculated parameters." }</p>
                    <p>{ "Suggestions and improvements are welcomed, feel free to open a ticket or pull request on " }<a href="https://github.com/rot256/probset">{ "Github" }</a>{"."}</p>
                    <div id="texts" style="display:inline; white-space:nowrap;">
                        {"Mathias Hall-Andersen <mathias"}
                    </div>
                    <div id="image" style="display:inline;">
                        <img src="at.svg"/>
                    </div>
                    <div id="texts" style="display:inline; white-space:nowrap;">
                        {"hall-andersen.dk>"}
                    </div>
                    </center>
                </footer>
            </div>
        }
    }
}
