import DatePicker from 'react-datepicker';
import { format, parse } from 'date-fns';
import { useSettings } from '../hooks/useSettings';
import {
  normalizeDate,
  normalizeTime,
  pickerDateTimeFormats,
  splitDatetimeLocal,
} from '../utils/datetime';
// react-datepicker.css is imported from main.tsx so Vite does not inject it
// asynchronously with the lazy form chunk (WebKitGTK paint glitches).

const INPUT_CLASS =
  'w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500';

/** Canonical wall-clock storage (always 24h), independent of display preference. */
const VALUE_FORMAT = "yyyy-MM-dd'T'HH:mm";
const PARSE_FORMAT = 'yyyy-MM-dd HH:mm';

export interface DateTimeLocalFieldsProps {
  value: string;
  onChange: (value: string) => void;
  required?: boolean;
  id?: string;
}

function parseWallDateTime(value: string): Date | null {
  const { date, time } = splitDatetimeLocal(value);
  if (!normalizeDate(date) || !normalizeTime(time)) {
    return null;
  }
  const parsed = parse(`${date} ${time}`, PARSE_FORMAT, new Date());
  return Number.isNaN(parsed.getTime()) ? null : parsed;
}

function formatWallDateTime(date: Date | null): string {
  if (!date || Number.isNaN(date.getTime())) {
    return '';
  }
  return format(date, VALUE_FORMAT);
}

/**
 * Single react-datepicker with date + time (no native datetime-local).
 * Value remains YYYY-MM-DDTHH:mm; display follows the 12/24h setting.
 */
export function DateTimeLocalFields({
  value,
  onChange,
  required = false,
  id,
}: DateTimeLocalFieldsProps) {
  const { settings } = useSettings();
  const formats = pickerDateTimeFormats(settings.use_24_hour_clock);

  return (
    <DatePicker
      id={id}
      selected={parseWallDateTime(value)}
      onChange={(date) => onChange(formatWallDateTime(date))}
      showTimeSelect
      timeIntervals={5}
      timeCaption="Time"
      timeFormat={formats.timeFormat}
      dateFormat={formats.dateFormat}
      placeholderText={formats.placeholder}
      required={required}
      isClearable={!required}
      shouldCloseOnSelect
      showPopperArrow={false}
      className={INPUT_CLASS}
      wrapperClassName="w-full"
      calendarClassName="dit-datepicker"
      autoComplete="off"
      aria-label="Date and time"
    />
  );
}
