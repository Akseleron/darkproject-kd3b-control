mod tests {
    use std::{
        cell::Cell,
        fmt,
        io::{self, BufRead, Cursor, Read, Write},
        rc::Rc,
    };

    use kd3b_device::{BusType, DiscoveredHidInterface};

    use crate::probe_cli::{ProbeOperation, ProbeSession};

    #[derive(Debug, Clone, Copy)]
    struct SyntheticError(&'static str);

    impl fmt::Display for SyntheticError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(self.0)
        }
    }

    struct FakeProbe {
        metadata: DiscoveredHidInterface,
        open_calls: Rc<Cell<usize>>,
        result: Result<(), SyntheticError>,
    }

    impl ProbeOperation for FakeProbe {
        type OpenError = SyntheticError;

        fn selected_metadata(&self) -> &DiscoveredHidInterface {
            &self.metadata
        }

        fn open_and_drop(self) -> Result<(), Self::OpenError> {
            self.open_calls.set(self.open_calls.get() + 1);
            self.result
        }
    }

    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("synthetic input failure"))
        }
    }

    impl BufRead for FailingReader {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            Err(io::Error::other("synthetic input failure"))
        }

        fn consume(&mut self, _amount: usize) {}
    }

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("synthetic output failure"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("synthetic output failure"))
        }
    }

    fn metadata() -> DiscoveredHidInterface {
        DiscoveredHidInterface {
            interface_number: 2,
            vendor_id: 0x195d,
            product_id: 0x2061,
            product_string: Some("Turing Gaming Keyboard".to_owned()),
            manufacturer_string: Some("Dark Project".to_owned()),
            serial_number: Some("selected-serial".to_owned()),
            release_number: 0x010a,
            bus_type: BusType::Usb,
            path: "presentation-only-path".to_owned(),
        }
    }

    fn fake_probe(open_calls: &Rc<Cell<usize>>) -> FakeProbe {
        FakeProbe {
            metadata: metadata(),
            open_calls: Rc::clone(open_calls),
            result: Ok(()),
        }
    }

    #[test]
    fn probe_opens_once_when_exact_confirmation_is_read() {
        // Given
        let open_calls = Rc::new(Cell::new(0));
        let mut input = Cursor::new(b"OPEN INTERFACE 2\n");
        let mut output = Vec::new();
        let session = ProbeSession::new(&mut input, &mut output, true);

        // When
        let result = session.run(|| Ok::<_, SyntheticError>(fake_probe(&open_calls)));

        // Then
        assert_eq!(result.exit_code, 0);
        assert_eq!(open_calls.get(), 1);
        let rendered = String::from_utf8(output).expect("probe output is UTF-8");
        assert!(rendered.contains("Interface: 2\n"));
        assert!(rendered.contains("Path: presentation-only-path\n"));
        assert!(rendered.contains(concat!(
            "No HID report read/write/feature operation was requested.\n",
            "This statement covers application-level HID report APIs only; the HIDAPI/libusb backend may exhibit the disclosed metadata/open/interface/interrupt behavior where applicable.\n",
        )));
    }

    #[test]
    fn probe_does_not_open_when_confirmation_is_wrong_or_eof() {
        // Given
        for confirmation in [b"OPEN INTERFACE 1\n".as_slice(), b"".as_slice()] {
            let open_calls = Rc::new(Cell::new(0));
            let mut input = Cursor::new(confirmation);
            let mut output = Vec::new();
            let session = ProbeSession::new(&mut input, &mut output, true);

            // When
            let result = session.run(|| Ok::<_, SyntheticError>(fake_probe(&open_calls)));

            // Then
            assert_eq!(result.exit_code, 1);
            assert_eq!(open_calls.get(), 0);
        }
    }

    #[test]
    fn probe_does_not_prepare_or_open_when_stdin_is_not_interactive() {
        // Given
        let prepare_calls = Cell::new(0);
        let open_calls = Rc::new(Cell::new(0));
        let mut input = Cursor::new(b"OPEN INTERFACE 2\n");
        let mut output = Vec::new();
        let session = ProbeSession::new(&mut input, &mut output, false);

        // When
        let result = session.run(|| {
            prepare_calls.set(prepare_calls.get() + 1);
            Ok::<_, SyntheticError>(fake_probe(&open_calls))
        });

        // Then
        assert_eq!(result.exit_code, 1);
        assert_eq!(prepare_calls.get(), 0);
        assert_eq!(open_calls.get(), 0);
    }

    #[test]
    fn probe_does_not_open_when_input_or_disclosure_output_fails() {
        // Given: an input failure after successful disclosure.
        let input_open_calls = Rc::new(Cell::new(0));
        let mut failing_input = FailingReader;
        let mut output = Vec::new();
        let input_session = ProbeSession::new(&mut failing_input, &mut output, true);

        // When
        let input_result =
            input_session.run(|| Ok::<_, SyntheticError>(fake_probe(&input_open_calls)));

        // Then
        assert_eq!(input_result.exit_code, 1);
        assert_eq!(input_open_calls.get(), 0);

        // Given: an output failure before confirmation.
        let output_open_calls = Rc::new(Cell::new(0));
        let mut input = Cursor::new(b"OPEN INTERFACE 2\n");
        let mut failing_output = FailingWriter;
        let output_session = ProbeSession::new(&mut input, &mut failing_output, true);

        // When
        let output_result =
            output_session.run(|| Ok::<_, SyntheticError>(fake_probe(&output_open_calls)));

        // Then
        assert_eq!(output_result.exit_code, 1);
        assert_eq!(output_open_calls.get(), 0);
    }

    #[test]
    fn probe_renders_typed_prepare_and_open_failures() {
        // Given: preparation fails before a prepared probe exists.
        let mut input = Cursor::new(b"OPEN INTERFACE 2\n");
        let mut output = Vec::new();
        let prepare_session = ProbeSession::new(&mut input, &mut output, true);

        // When
        let prepare_result =
            prepare_session.run(|| Err::<FakeProbe, _>(SyntheticError("selection failed")));

        // Then
        assert_eq!(prepare_result.exit_code, 1);
        assert_eq!(
            prepare_result.stderr,
            "error: could not prepare configuration-interface probe: selection failed\n"
        );

        // Given: opening the confirmed selected probe fails.
        let open_calls = Rc::new(Cell::new(0));
        let mut input = Cursor::new(b"OPEN INTERFACE 2\n");
        let mut output = Vec::new();
        let open_session = ProbeSession::new(&mut input, &mut output, true);

        // When
        let open_result = open_session.run(|| {
            Ok::<_, SyntheticError>(FakeProbe {
                metadata: metadata(),
                open_calls: Rc::clone(&open_calls),
                result: Err(SyntheticError("selected open failed")),
            })
        });

        // Then
        assert_eq!(open_result.exit_code, 1);
        assert_eq!(open_calls.get(), 1);
        assert_eq!(
            open_result.stderr,
            "error: could not open selected configuration interface: selected open failed\n"
        );
    }
}
