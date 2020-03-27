use super::FilterParameters;

#[derive(Clone, Copy, Debug)]
pub struct Parameters {
    error: Option<f64>,    // false positive rate
    elements: Option<f64>, // number of elements
    storage: Option<f64>,  // storage (bits)
    hashes: Option<f64>,   // fingerprint size
}

impl FilterParameters for Parameters {
    fn error(&self) -> Option<f64> {
        self.error
    }

    fn elements(&self) -> Option<u64> {
        self.elements.map(|v| v as u64)
    }

    fn storage(&self) -> Option<u64> {
        self.storage.map(|v| v as u64)
    }

    fn bits_per_element(&self) -> Option<f64> {
        self.storage
            .and_then(|storage| self.elements.map(|elements| storage / elements))
    }
}

impl Parameters {
    pub fn new(
        error: Option<f64>,
        elements: Option<u64>,
        storage: Option<u64>,
        hashes: Option<u64>,
    ) -> Parameters {
        let param = Parameters {
            error,
            elements: elements.map(|v| v as f64),
            storage: storage.map(|v| v as f64),
            hashes: hashes.map(|v| v as f64),
        };

        param.infer()
    }

    fn infer(mut self) -> Parameters {
        for _ in 0..4 {
            self.storage.map(|storage| {
                self.error.map(|error| {
                    let c = f64::ln(2.0) * f64::ln(2.0);
                    self.elements = Some(f64::floor(-(storage * c / f64::ln(error))));
                })
            });

            self.elements.map(|elements| {
                self.error.map(|error| {
                    if self.storage.is_none() {
                        let c = f64::ln(2.0) * f64::ln(2.0);
                        self.storage = Some(f64::ceil(-elements * f64::ln(error) / c));
                    };
                });

                self.storage.map(|storage| {
                    if self.hashes.is_none() {
                        self.error = None; // affects the error
                        self.hashes = Some(f64::round(storage / elements * f64::ln(2.0)));
                    };

                    if self.error.is_none() {
                        let c = f64::ln(2.0) * f64::ln(2.0);
                        self.storage = None; // affects storage
                        self.error = Some((-(storage * c) / elements).exp());
                    }
                });
            });
        }

        self
    }

    pub fn hashes(&self) -> Option<u64> {
        self.hashes.map(|v| v as u64)
    }
}
