use super::FilterParameters;

#[derive(Clone, Copy, Debug)]
pub struct Parameters {
    error: Option<f64>,    // false positive rate
    elements: Option<f64>, // number of elements
    storage: Option<f64>,  // storage (bits)
    bits: Option<f64>,
}

impl FilterParameters for Parameters {
    fn bits_per_element(&self) -> Option<f64> {
        self.bits
    }

    fn error(&self) -> Option<f64> {
        self.error
    }

    fn elements(&self) -> Option<u64> {
        self.elements.map(|v| v as u64)
    }

    fn storage(&self) -> Option<u64> {
        self.storage.map(|v| v as u64)
    }
}

impl Parameters {
    pub fn new(error: Option<f64>, elements: Option<u64>, storage: Option<u64>) -> Parameters {
        let mut contraints = 0;

        contraints += error.is_some() as u32;
        contraints += elements.is_some() as u32;
        contraints += storage.is_some() as u32;

        let params = Parameters {
            error,
            elements: elements.map(|v| v as f64),
            storage: storage.map(|v| v as f64),
            bits: None,
        };

        if contraints == 2 {
            params.infer()
        } else {
            params
        }
    }

    fn infer(mut self) -> Parameters {
        for _ in 0..2 {
            self.bits = self
                .bits
                .or_else(|| self.error.and_then(|error| Some(f64::log2(1.0 / error))));

            self.bits = self.bits.or_else(|| {
                self.storage
                    .and_then(|storage| self.elements.and_then(|elements| Some(storage / elements)))
            });

            self.bits.map(|bits| {
                self.storage = self
                    .storage
                    .or_else(|| self.elements.and_then(|elements| Some(bits * elements)));

                self.elements = self
                    .elements
                    .or_else(|| self.storage.and_then(|storage| Some(storage / bits)));

                self.error = self.error.or_else(|| Some(f64::powf(2.0, -bits)));
            });
        }

        self
    }
}
