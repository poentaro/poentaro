//! The single `impl Guest for Component` block (Rust forbids splitting
//! one trait impl across files). Each method's real logic — and the
//! `PLAN-wit-bindgen-migration.md` "Phase N" comment describing it — lives
//! in the same-named function in the relevant `guest_*` submodule; this
//! file is just the trait wiring.
use super::*;

/// Trait methods whose body is just a call to the free function of the same
/// name and arguments.
macro_rules! forward_to_free_functions {
    ($($name:ident($($arg:ident: $ty:ty),*) -> $ret:ty;)*) => {
        $(fn $name($($arg: $ty),*) -> $ret {
            $name($($arg),*)
        })*
    };
}

impl Guest for Component {
    fn greet(name: String) -> String {
        greet(name)
    }

    fn group_note_selection(
        note_spans: Vec<NoteSpan>,
        selected_cells: Vec<NoteCellIn>,
    ) -> GroupNoteSelectionResponse {
        group_note_selection(note_spans, selected_cells)
    }

    fn group_lyric_selection(
        lyric_spans: Vec<LyricSpan>,
        selected_cells: Vec<LyricCellIn>,
    ) -> GroupLyricSelectionResponse {
        group_lyric_selection(lyric_spans, selected_cells)
    }

    fn list_measure_spans(source: String) -> ListMeasureSpansResponse {
        list_measure_spans(source)
    }

    fn list_note_spans(
        source: String,
        enabled_tracks: Option<Vec<String>>,
    ) -> ListNoteSpansResponse {
        list_note_spans(source, enabled_tracks)
    }

    fn list_lyric_spans(
        source: String,
        enabled_tracks: Option<Vec<String>>,
    ) -> ListLyricSpansResponse {
        list_lyric_spans(source, enabled_tracks)
    }

    fn list_parts(source: String, raw_instruments: Vec<InstrumentInfo>) -> ListPartsResponse {
        list_parts(source, raw_instruments)
    }

    fn list_symbols(source: String, raw_instruments: Vec<InstrumentInfo>) -> ListSymbolsResponse {
        list_symbols(source, raw_instruments)
    }

    fn rename_symbol(
        source: String,
        kind: SymbolKind,
        old_name: String,
        new_name: String,
        raw_instruments: Vec<InstrumentInfo>,
    ) -> RenameSymbolResponse {
        rename_symbol(source, kind, old_name, new_name, raw_instruments)
    }

    fn get_measure_index_at_offset(source: String, byte_offset: u32) -> MeasureAtOffsetResponse {
        get_measure_index_at_offset(source, byte_offset)
    }

    fn resolve_part_visibility(
        source: String,
        parts: Vec<Part>,
        toggles: PartToggleState,
        preset_name: Option<String>,
    ) -> ResolvedPartVisibility {
        resolve_part_visibility(source, parts, toggles, preset_name)
    }

    fn render_svg(
        source: String,
        visibility: ResolvedPartVisibility,
        raw_instruments: Vec<InstrumentInfo>,
        note_labels: NoteLabels,
    ) -> RenderResponse {
        render_svg(source, visibility, raw_instruments, note_labels)
    }

    fn render_mei(
        source: String,
        visibility: ResolvedPartVisibility,
        raw_instruments: Vec<InstrumentInfo>,
        clef: StaffClef,
    ) -> RenderMeiResponse {
        render_mei(source, visibility, raw_instruments, clef)
    }

    fn generate_mp3(
        source: String,
        enabled_tracks: Option<Vec<String>>,
        soundfont: Vec<u8>,
    ) -> GenerateMp3Response {
        generate_mp3(source, enabled_tracks, soundfont)
    }

    fn generate_pdf(
        source: String,
        visibility: ResolvedPartVisibility,
        fonts: FontBytesByFamily,
        note_labels: NoteLabels,
    ) -> GeneratePdfResponse {
        generate_pdf(source, visibility, fonts, note_labels)
    }

    fn generate_midi(source: String, enabled_tracks: Option<Vec<String>>) -> GenerateMidiResponse {
        generate_midi(source, enabled_tracks)
    }

    fn generate_mp3_for_measure_range(
        source: String,
        start_index: u32,
        end_index: u32,
        extend_to_last_occurrence: bool,
        respect_sequence: bool,
        sequence_entry_start_index: Option<u32>,
        sequence_entry_end_index: Option<u32>,
        enabled_tracks: Option<Vec<String>>,
        trim_start_s: Option<f64>,
        trim_end_s: Option<f64>,
        trim_next_note_start_s: Option<f64>,
        soundfont: Vec<u8>,
    ) -> GenerateMp3Response {
        generate_mp3_for_measure_range(
            source,
            start_index,
            end_index,
            extend_to_last_occurrence,
            respect_sequence,
            sequence_entry_start_index,
            sequence_entry_end_index,
            enabled_tracks,
            trim_start_s,
            trim_end_s,
            trim_next_note_start_s,
            soundfont,
        )
    }

    fn list_note_timings(
        source: String,
        visible_tracks: Option<Vec<String>>,
        enabled_tracks: Option<Vec<String>>,
    ) -> NoteTimingsResponse {
        list_note_timings(source, visible_tracks, enabled_tracks)
    }

    fn list_note_timings_for_range(
        source: String,
        start_index: u32,
        end_index: u32,
        extend_to_last_occurrence: bool,
        respect_sequence: bool,
        sequence_entry_start_index: Option<u32>,
        sequence_entry_end_index: Option<u32>,
        visible_tracks: Option<Vec<String>>,
        enabled_tracks: Option<Vec<String>>,
    ) -> NoteTimingsResponse {
        list_note_timings_for_range(
            source,
            start_index,
            end_index,
            extend_to_last_occurrence,
            respect_sequence,
            sequence_entry_start_index,
            sequence_entry_end_index,
            visible_tracks,
            enabled_tracks,
        )
    }

    fn generate_instrument_preview_wav(
        program_number: u8,
        soundfont: Vec<u8>,
    ) -> GenerateWavResponse {
        generate_instrument_preview_wav(program_number, soundfont)
    }

    fn generate_percussion_preview_wav(key: u8, soundfont: Vec<u8>) -> GenerateWavResponse {
        generate_percussion_preview_wav(key, soundfont)
    }

    fn parse_metadata_fields(source: String, preset_name: Option<String>) -> MetadataFields {
        parse_metadata_fields(source, preset_name)
    }

    fn update_metadata_field(
        source: String,
        preset_name: Option<String>,
        edit: MetadataEdit,
    ) -> String {
        update_metadata_field(source, preset_name, edit)
    }

    fn highlight_tokens(source: String) -> Vec<HighlightToken> {
        highlight_tokens(source)
    }

    fn section_headers(source: String) -> Vec<SectionHeader> {
        section_headers(source)
    }

    forward_to_free_functions! {
        select_notes_in_byte_ranges(note_spans: Vec<NoteSpan>, ranges: Vec<ByteRange>) -> NoteSelection;
        select_lyrics_in_byte_ranges(lyric_spans: Vec<LyricSpan>, ranges: Vec<ByteRange>) -> LyricSelection;
        note_selection_trim_window(cells: Vec<NoteCellIn>, note_timings: Vec<NoteTiming>) -> Option<TrimWindow>;
        resolve_system_selection_range(note_spans: Vec<NoteSpan>, lyric_spans: Vec<LyricSpan>, systems: Vec<MeasureRangeIn>, anchor: ClickableElementId, current: ClickableElementId) -> ResolveSelectionRangeResponse;
        fully_selected_labels(note_spans: Vec<NoteSpan>, lyric_spans: Vec<LyricSpan>, labels: Vec<ClickableElementId>, note_cells: Vec<NoteCellIn>, lyric_cells: Vec<LyricCellIn>) -> Vec<bool>;
        list_presets(source: String) -> Vec<PresetSummary>;
        update_preset_field(source: String, preset_name: String, edit: PresetEdit) -> String;
        set_preset_part_shown(source: String, preset_name: String, all_parts: Vec<String>, part: PartShown) -> String;
        set_preset_part_soloed(source: String, preset_name: String, all_parts: Vec<String>, part: PartSoloed) -> String;
        new_preset_name_error(source: String, name: String) -> Option<String>;
        add_preset_section(source: String, preset: NewPreset) -> String;
        follow_preset_rename(previous_source: String, source: String, preset_name: String) -> Option<String>;
        staff_measure_at_point(measures: Vec<StaffMeasureGeometry>, x: f64, y: f64) -> Option<u32>;
        staff_measure_area(measure: StaffMeasureGeometry) -> BoundingBox;
        highlighted_staff_measures(measure_indices: Vec<u32>, ranges: Vec<MeasureRangeIn>) -> Vec<u32>;
        pending_staff_measures(measure_indices: Vec<u32>, anchor: MeasureRangeIn, current: MeasureRangeIn) -> Vec<u32>;
    }

    fn set_layout_fonts(fonts: FontBytesByFamily) {
        set_layout_fonts(fonts)
    }

    fn shift_part_octave(source: String, abbreviation: String, delta: i32) -> String {
        shift_part_octave(source, abbreviation, delta)
    }

    fn list_streamable_parts(source: String) -> Vec<StreamablePart> {
        list_streamable_parts(source)
    }

    fn extract_part_stream(source: String, abbreviation: String) -> ExtractPartStreamResponse {
        extract_part_stream(source, abbreviation)
    }

    fn shift_range_octave(source: String, ranges: Vec<ByteRange>, delta: i32) -> RangeEditResponse {
        shift_range_octave(source, ranges, delta)
    }

    fn toggle_range_slur(source: String, ranges: Vec<ByteRange>) -> RangeEditResponse {
        toggle_range_slur(source, ranges)
    }

    fn toggle_range_tie(source: String, ranges: Vec<ByteRange>) -> RangeEditResponse {
        toggle_range_tie(source, ranges)
    }

    fn format_score(source: String) -> String {
        format_score(source)
    }

    fn describe_selection(source: String, ranges: Vec<ByteRange>) -> Option<PitchDescription> {
        describe_selection(source, ranges)
    }

    fn list_part_declarations(
        source: String,
        raw_instruments: Vec<InstrumentInfo>,
    ) -> ListPartDeclarationsResponse {
        list_part_declarations(source, raw_instruments)
    }

    fn update_part_declaration(
        source: String,
        abbreviation: String,
        settings: PartSettings,
    ) -> String {
        update_part_declaration(source, abbreviation, settings)
    }

    fn list_sound_choices(raw_instruments: Vec<InstrumentInfo>) -> SoundChoices {
        list_sound_choices(raw_instruments)
    }

    fn list_range_instruments() -> Vec<RangeInstrumentChoice> {
        list_range_instruments()
    }

    fn get_part_setting_limits() -> PartSettingLimits {
        get_part_setting_limits()
    }

    fn extract_source_from_svg(svg_bytes: Vec<u8>) -> Option<String> {
        extract_source_from_svg(svg_bytes)
    }

    fn extract_source_from_pdf(pdf_bytes: Vec<u8>) -> Option<String> {
        extract_source_from_pdf(pdf_bytes)
    }

    fn compress_share_payload(payload: String) -> Vec<u8> {
        compress_share_payload(payload)
    }

    fn decompress_share_payload(bytes: Vec<u8>) -> Option<String> {
        decompress_share_payload(bytes)
    }

    fn resolve_selection_range(
        note_spans: Vec<NoteSpan>,
        lyric_spans: Vec<LyricSpan>,
        anchor: ClickableElementId,
        current: ClickableElementId,
    ) -> ResolveSelectionRangeResponse {
        resolve_selection_range(note_spans, lyric_spans, anchor, current)
    }

    forward_to_free_functions! {
        apply_part_stream(source: String, abbreviation: String, edited_stream: String) -> PartStreamApplyResult;
        format_part_stream(source: String, abbreviation: String, edited_stream: String, cursors: Vec<u32>) -> FormattedPartStream;
        part_focus_measures(source: String, abbreviation: String, toggles: PartToggleState) -> PartFocusMeasuresResponse;
        part_stream_ranges_to_source(source: String, abbreviation: String, edited_stream: String, ranges: Vec<Span>) -> Vec<Span>;
        source_ranges_to_part_stream(source: String, abbreviation: String, edited_stream: String, ranges: Vec<Span>) -> Vec<Span>;
        render_svg_with_highlight_range(source: String, raw_measure_ranges: Vec<MeasureRangeIn>, visibility: ResolvedPartVisibility, raw_instruments: Vec<InstrumentInfo>, note_labels: NoteLabels) -> RenderResponse;
        render_split_meis(source: String, base_name: String, preset_name: Option<String>, raw_instruments: Vec<InstrumentInfo>, clef: StaffClef) -> RenderSplitMeisResponse;
        generate_staff_pdf(pages: Vec<String>, source: String, fonts: FontBytesByFamily) -> GeneratePdfResponse;
        generate_split_staff_pdfs(parts: Vec<StaffPdfPart>, fonts: FontBytesByFamily) -> GenerateSplitPdfsResponse;
        generate_split_pdfs(source: String, base_name: String, preset_name: Option<String>, fonts: FontBytesByFamily, note_labels: NoteLabels) -> GenerateSplitPdfsResponse;
        generate_split_midis(source: String, base_name: String, preset_name: Option<String>) -> GenerateSplitMidisResponse;
        generate_split_wavs(source: String, base_name: String, preset_name: Option<String>, soundfont: Vec<u8>) -> GenerateSplitWavsResponse;
        generate_split_mp3s(source: String, base_name: String, preset_name: Option<String>, soundfont: Vec<u8>) -> GenerateSplitMp3sResponse;
        generate_wav(source: String, enabled_tracks: Option<Vec<String>>, metronome: bool, soundfont: Vec<u8>) -> GenerateWavResponse;
        generate_wav_for_measure_range(source: String, start_index: u32, end_index: u32, extend_to_last_occurrence: bool, respect_sequence: bool, sequence_entry_start_index: Option<u32>, sequence_entry_end_index: Option<u32>, enabled_tracks: Option<Vec<String>>, metronome: bool, trim_start_s: Option<f64>, trim_end_s: Option<f64>, trim_next_note_start_s: Option<f64>, soundfont: Vec<u8>) -> GenerateWavResponse;
        file_store_create_folder(state: FileStoreState, parent: Option<String>, name: String, context: FileOperationContext) -> FileStoreResponse;
        file_store_create_folder_error(state: FileStoreState, parent: Option<String>, name: String, context: FileOperationContext) -> Option<String>;
        file_store_move_targets(state: FileStoreState, excluded_folder: Option<String>) -> Vec<MoveTarget>;
        file_store_move_destination(source_path: String, folder: Option<String>) -> Option<String>;
        file_store_name_after_collision(rejected: String) -> Option<String>;
        file_store_classify_cloud_listing(entries: Vec<CloudListedEntry>) -> CloudListingClassification;
        file_store_placeholder_path(folder: String) -> Option<String>;
        file_store_is_placeholder_path(path: String) -> bool;
        file_store_rename_error(state: FileStoreState, from_path: String, to_path: String, context: FileOperationContext) -> Option<String>;
        file_store_folder_move_error(state: FileStoreState, from_path: String, to_path: String, context: FileOperationContext) -> Option<String>;
        file_store_folder_delete_summary(state: FileStoreState, folder: String, shares: ShareOverview) -> Option<FolderDeleteSummary>;
        file_store_folder_share_info(state: FileStoreState, shares: ShareOverview, folder: String) -> Option<FolderShareInfo>;
        file_store_folder_static_share(state: FileStoreState, folder: String) -> Option<FolderStaticShare>;
        decode_folder_share_suffix(suffix: String) -> Option<FolderSnapshot>;
        shared_folder_first_file(snapshot: FolderSnapshot) -> Option<String>;
        shared_folder_view(snapshot: FolderSnapshot, active: String, expanded: Vec<String>) -> FileTreeViewResponse;
        file_store_import_shared_folder(state: FileStoreState, snapshot: FolderSnapshot, file_ids: Vec<String>, context: FileOperationContext) -> FileStoreResponse;
        file_store_bin_entries(state: FileStoreState) -> Vec<BinEntry>;
        file_store_folders_to_reveal(path: String) -> Vec<String>;
        file_store_create_file(state: FileStoreState, folder: Option<String>, name: String, content: String, context: FileOperationContext) -> FileStoreResponse; file_store_create_file_error(state: FileStoreState, folder: Option<String>, name: String, context: FileOperationContext) -> Option<String>;
        file_store_duplicate_file(state: FileStoreState, source: String, context: FileOperationContext) -> FileStoreResponse;
        file_store_merge_migrated(current: FileStoreState, carried: FileStoreState, context: FileOperationContext) -> FileStoreResponse;
        file_store_rename_legacy_extensions(state: FileStoreState, context: FileOperationContext) -> Option<FileStoreState>;
        file_store_rename_file(state: FileStoreState, from_path: String, to_path: String, context: FileOperationContext) -> FileStoreResponse;
        file_store_delete_file(state: FileStoreState, path: String, context: FileOperationContext) -> FileStoreResponse;
        file_store_restore_file(state: FileStoreState, bin_key: String, context: FileOperationContext) -> FileStoreResponse;
        file_store_move_folder(state: FileStoreState, from_path: String, to_path: String, context: FileOperationContext) -> FolderMoveResponse;
        file_store_delete_folder(state: FileStoreState, folder: String, context: FileOperationContext) -> FileStoreResponse;
        file_store_tree(state: FileStoreState) -> FileTreeResponse;
        file_store_tree_view(state: FileStoreState, expanded: Vec<String>, demo_file_names: Vec<String>, shares: ShareOverview, query: String, selected: Vec<String>) -> FileTreeViewResponse; file_store_selection_all(state: FileStoreState) -> Vec<String>; file_store_selection_toggle(state: FileStoreState, selected: Vec<String>, key: String) -> Vec<String>; file_store_selection_units(state: FileStoreState, selected: Vec<String>) -> Vec<SelectionUnit>; file_store_selection_move_plan(state: FileStoreState, selected: Vec<String>, folder: Option<String>, context: FileOperationContext) -> SelectionMovePlan; file_store_selection_delete_summary(state: FileStoreState, selected: Vec<String>, shares: ShareOverview) -> SelectionDeleteSummary;
        file_store_active_label(active: String, demo_file_names: Vec<String>) -> FileActiveLabel;
        outbox_enqueue(queue: Queue, file_id: String, message: Message, message_id: String, now_ms: u64) -> Queue;
        outbox_begin_send(queue: Queue, now_ms: u64) -> BeginSend;
        outbox_next_wake_ms(queue: Queue) -> Option<u64>;
        outbox_on_result(queue: Queue, file_id: String, message_id: String, send_result: SendResult, now_ms: u64, jitter_unit: f64) -> Queue;
        outbox_resolve(queue: Queue, file_id: String, resolution: Resolution) -> Queue;
        outbox_resolve_signed_in(queue: Queue) -> Queue;
        outbox_record_revision(queue: Queue, file_id: String, revision: i64) -> Queue;
        outbox_encode_queue(queue: Queue) -> Vec<StoredRecord>;
        outbox_decode_queue(records: Vec<StoredRecord>) -> Result<Queue, DecodeError>;
        outbox_summarize(queue: Queue, now_ms: u64) -> QueueSummary;
        outbox_record_sent(queue_before: Queue, existing: Vec<StoredRecord>, file_id: String, message_id: String, revision: i64, now_ms: u64) -> HistoryWrite;
        outbox_history_rows(queue: Queue, sent_records: Vec<StoredRecord>, file_names: Vec<FileName>, filter: HistoryFilter, now_ms: u64) -> Vec<HistoryRow>;
    }

    fn merge_three_way(base: String, mine: String, theirs: String) -> MergeOutcome {
        merge_three_way(base, mine, theirs)
    }
}
