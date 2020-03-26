#[derive(Clone, Copy, Debug)]
pub struct Parameters {
    // user defined / infered
    error: Option<f64>,    // false positive rate
    elements: Option<f64>, // number of elements
    storage: Option<f64>,  // storage (bits)

    // infered optimal values
    fingerprint: Option<f64>, // fingerprint size
    buckets: Option<f64>,     // buckets in map

    // hyper parameters
    hashes: f64, // possible buckets for each element (normally 2)
    slots: f64,  // slots per bucket (e.g. 4)
    util: f64,   // utilization, util \in (0, 1]
}

impl Parameters {
    /// Infers new parameters from contraints on:
    ///
    /// - error: the false positive rate.
    /// - elements: the number of elements to store.
    /// - storage: the number of bits for the filter.
    ///
    /// At most 2 of which may be supplied,
    /// otherwise the system is over constrained.
    ///
    /// Additionally the following hyper parameters must be supplied:
    ///
    /// - hashes: possible bucket for each element (normally 2)
    /// - slots: slots per bucket (e.g. 4)
    /// - util: utilization (usually >=0.95 to compete with Bloom filters)
    ///
    /// # Returns
    ///
    /// A full resolved set of optimal parameters.
    pub fn new(
        error: Option<f64>,
        elements: Option<u64>,
        storage: Option<u64>,
        hashes: u64,
        slots: u64,
        util: f64,
    ) -> Parameters {
        let mut contraints = 0;

        contraints += error.is_some() as u32;
        contraints += elements.is_some() as u32;
        contraints += storage.is_some() as u32;

        let params = Parameters {
            error,
            elements: elements.map(|v| v as f64),
            storage: storage.map(|v| v as f64),
            fingerprint: None,
            buckets: None,
            hashes: hashes as f64,
            slots: slots as f64,
            util,
        };

        if contraints == 2 {
            params.infer()
        } else {
            params
        }
    }

    pub fn error(&self) -> Option<f64> {
        self.error
    }

    pub fn elements(&self) -> Option<u64> {
        self.elements.map(|v| v as u64)
    }

    pub fn storage(&self) -> Option<u64> {
        self.storage.map(|v| v as u64)
    }

    pub fn fingerprint(&self) -> Option<u64> {
        self.fingerprint.map(|v| v as u64)
    }

    pub fn buckets(&self) -> Option<u64> {
        self.buckets.map(|v| v as u64)
    }

    pub fn slots(&self) -> u64 {
        self.slots as u64
    }

    pub fn util(&self) -> f64 {
        self.util
    }

    pub fn bits_per_element(&self) -> Option<f64> {
        self.storage
            .and_then(|storage| self.elements.map(|elements| storage / elements))
    }

    fn incomplete(&self) -> bool {
        self.buckets.is_none()
            || self.error.is_none()
            || self.fingerprint.is_none()
            || self.storage.is_none()
            || self.elements.is_none()
    }

    fn infer(mut self) -> Parameters {
        for _ in 0..8 {
            if !self.incomplete() {
                break;
            }

            // Infer fingerprint size, from:
            //   - storage
            //   - elements
            //   - util
            //
            //
            self.fingerprint = self.fingerprint.or_else(|| {
                self.elements.and_then(|elements| {
                    self.storage
                        .and_then(|storage| Some(f64::floor((storage * self.util) / elements)))
                })
            });

            // Infer fingerprint size, from:
            //  - error
            //  - util
            //  - slots
            //  - hashes
            self.fingerprint = self.fingerprint.or_else(|| {
                self.error.and_then(|error| {
                    self.error = None; // this affects the error (may decrease)
                    Some(f64::ceil(f64::log2(
                        self.util * self.slots * self.hashes / error,
                    )))
                })
            });

            self.fingerprint.map(|fingerprint| {
                // Infer false positive rate, from:
                //  - size of fingerprint
                //  - number of slots per bucket
                //  - utilization
                self.error = self.error.or_else(|| {
                    let ok_one = 1. - f64::powf(2.0, -fingerprint);
                    let ok_all = ok_one.powf(self.slots * self.hashes * self.util);
                    Some(1. - ok_all)
                });

                // Infer buckets, from:
                //  - size of fingerprint
                //  - elements to store
                //  - util
                self.buckets = self.buckets.or_else(|| {
                    self.elements.and_then(|elements| {
                        let cells = f64::ceil(elements / self.util);
                        Some(f64::ceil(cells / self.slots))
                    })
                });

                // Infer number of buckets, from:
                //  - size of fingerprint
                //  - total storage
                //  - slots per bucket
                self.buckets = self.buckets.or_else(|| {
                    self.storage.and_then(|storage| {
                        let cells = f64::floor(storage / fingerprint);
                        Some(f64::floor(cells / self.slots))
                    })
                });

                // Infer storage, from:
                //  - size of fingerprint
                //  - slots per bucket
                //  - number of buckets
                self.storage = self.storage.or_else(|| {
                    self.buckets.and_then(|buckets| {
                        let cells = buckets * self.slots;
                        Some(cells * fingerprint)
                    })
                });
            });

            // Infer number of stored elements, from:
            //  - number of buckets
            //  - utilization
            self.elements = self.elements.or_else(|| {
                self.buckets.and_then(|buckets| {
                    let cells = buckets * self.slots;
                    Some(f64::floor(cells * self.util))
                })
            })
        }

        self
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference() {
        let error = 0.099 / 100.;
        let elements = 128040_000_000;
        let param = Parameters::new(Some(error), Some(elements), None, 2, 4, 0.95).unwrap();
        assert!(param.error().unwrap() <= error);
        assert!(param.elements().unwrap() >= elements);

        // obtained from SageMath implementation
        assert_eq!(param.fingerprint().unwrap(), 13);
    }
}
*/
