# Cron Components

A comprehensive set of React components for creating, editing, and displaying cron expressions with TypeScript support and shadcn/ui integration.

## Features

- 🎯 **Multiple Input Modes**: Text input, visual builder, or both
- 🔍 **Real-time Validation**: Powered by `@datasert/cronjs-parser`
- ⏰ **Next Run Preview**: Shows upcoming execution times with `@datasert/cronjs-matcher`
- 🌍 **Timezone Support**: Display times in different timezones
- 📱 **Responsive Design**: Works seamlessly on desktop and mobile
- 🎨 **shadcn/ui Integration**: Consistent with your existing design system
- 🔧 **TypeScript Support**: Full type definitions included
- 🚀 **Customizable**: Extensive props for customization

## Components

### `<Cron />` - Main Component

The primary component that combines all functionality.

```tsx
import { Cron } from '@/components/cron';

function MyScheduler() {
  const [cronExpression, setCronExpression] = useState('0 0 9 * * 1-5');

  return (
    <Cron
      value={cronExpression}
      onChange={setCronExpression}
      mode="both" // 'input' | 'builder' | 'both'
      showPreview={true}
      showDescription={true}
      timezone="UTC"
    />
  );
}
```

#### Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `value` | `string` | `''` | Current cron expression |
| `onChange` | `(value: string) => void` | - | Called when expression changes |
| `onValidate` | `(isValid: boolean) => void` | - | Called when validation state changes |
| `mode` | `'input' \| 'builder' \| 'both'` | `'both'` | Display mode |
| `disabled` | `boolean` | `false` | Disable all inputs |
| `placeholder` | `string` | `'0 0 * * * *'` | Input placeholder text |
| `showPreview` | `boolean` | `true` | Show next run times preview |
| `showDescription` | `boolean` | `true` | Show human-readable description |
| `timezone` | `string` | `'UTC'` | Timezone for preview times |
| `error` | `string` | - | External error message |
| `className` | `ClassValue` | - | Additional CSS classes |

### `<CronInput />` - Text Input Component

Simple text input with validation and help text.

```tsx
import { CronInput } from '@/components/cron';

function QuickEntry() {
  const [expression, setExpression] = useState('');
  const [isValid, setIsValid] = useState(false);

  return (
    <CronInput
      value={expression}
      onChange={setExpression}
      onValidate={setIsValid}
      placeholder="Enter cron expression..."
    />
  );
}
```

#### Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `value` | `string` | - | Current expression value |
| `onChange` | `(value: string) => void` | - | Called when input changes |
| `onValidate` | `(isValid: boolean) => void` | - | Called when validation changes |
| `placeholder` | `string` | `'0 0 * * * *'` | Placeholder text |
| `disabled` | `boolean` | `false` | Disable input |
| `readOnly` | `boolean` | `false` | Make input read-only |
| `error` | `string` | - | Error message to display |
| `className` | `ClassValue` | - | Additional CSS classes |

### `<CronBuilder />` - Visual Builder Component

Visual interface for building cron expressions with presets and field editors.

```tsx
import { CronBuilder } from '@/components/cron';

function VisualScheduler() {
  const [expression, setExpression] = useState('0 0 * * * *');

  return (
    <CronBuilder
      value={expression}
      onChange={setExpression}
      showPreview={true}
      defaultTab="daily"
      allowedPeriods={['hourly', 'daily', 'weekly']}
    />
  );
}
```

#### Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `value` | `string` | `'0 0 * * * *'` | Current expression |
| `onChange` | `(value: string) => void` | - | Called when expression changes |
| `disabled` | `boolean` | `false` | Disable all controls |
| `showPreview` | `boolean` | `true` | Show preview section |
| `defaultTab` | `CronPeriod` | `'hourly'` | Default active tab |
| `allowedPeriods` | `CronPeriod[]` | All periods | Which tabs to show |
| `presets` | `CronPreset[]` | Built-in presets | Custom preset list |
| `className` | `ClassValue` | - | Additional CSS classes |

### `<CronDisplay />` - Display Component

Read-only component for displaying cron expression information.

```tsx
import { CronDisplay } from '@/components/cron';

function ScheduleInfo({ schedule }) {
  return (
    <CronDisplay
      expression={schedule.cronExpression}
      showNextRuns={true}
      showDescription={true}
      nextRunsCount={5}
      timezone={schedule.timezone}
    />
  );
}
```

#### Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `expression` | `string` | - | Cron expression to display |
| `showNextRuns` | `boolean` | `true` | Show upcoming run times |
| `showDescription` | `boolean` | `true` | Show human-readable description |
| `nextRunsCount` | `number` | `5` | Number of future runs to show |
| `timezone` | `string` | `'UTC'` | Timezone for times |
| `className` | `ClassValue` | - | Additional CSS classes |

## Cron Expression Format

The components support 6-field cron expressions with seconds:

```
┌─────────────── second (0-59)
│ ┌───────────── minute (0-59)
│ │ ┌─────────── hour (0-23)
│ │ │ ┌───────── day of month (1-31)
│ │ │ │ ┌─────── month (1-12)
│ │ │ │ │ ┌───── day of week (0-6, Sunday=0)
│ │ │ │ │ │
* * * * * *
```

### Special Characters

| Character | Description | Example |
|-----------|-------------|---------|
| `*` | Any value | `*` = every value |
| `,` | List separator | `1,3,5` = values 1, 3, and 5 |
| `-` | Range | `1-5` = values 1 through 5 |
| `/` | Step values | `*/5` = every 5th value |
| `?` | No specific value | Used when day/weekday conflict |
| `L` | Last | Last day of month/week |
| `W` | Weekday | Nearest weekday |

### Common Examples

| Expression | Description |
|------------|-------------|
| `0 * * * * *` | Every minute |
| `0 */5 * * * *` | Every 5 minutes |
| `0 0 * * * *` | Every hour |
| `0 0 9 * * *` | Daily at 9 AM |
| `0 30 9 * * 1-5` | Weekdays at 9:30 AM |
| `0 0 0 * * 0` | Every Sunday at midnight |
| `0 0 0 1 * *` | First day of every month |
| `0 0 0 1 1 *` | Every January 1st |

## Dependencies

- `@datasert/cronjs-parser` - For parsing and validating cron expressions
- `@datasert/cronjs-matcher` - For calculating next run times
- `@radix-ui/react-*` - UI primitives (via shadcn/ui)
- `lucide-react` - Icons

## Installation

1. Copy the component files to your project
2. Ensure you have the required dependencies:

```bash
npm install @datasert/cronjs-parser @datasert/cronjs-matcher
```

3. Import and use the components:

```tsx
import { Cron } from '@/components/cron';
```

## Customization

### Custom Presets

```tsx
const customPresets = [
  {
    label: 'Business Hours',
    value: '0 0 9-17 * * 1-5',
    description: 'Every hour during business hours',
    category: 'custom'
  },
  // ... more presets
];

<CronBuilder presets={customPresets} />
```

### Restricted Periods

```tsx
<CronBuilder
  allowedPeriods={['daily', 'weekly']}
  defaultTab="daily"
/>
```

### Custom Validation

```tsx
function MyComponent() {
  const [expression, setExpression] = useState('');
  const [isValid, setIsValid] = useState(false);

  const handleValidation = (valid: boolean) => {
    setIsValid(valid);
    // Custom validation logic
  };

  return (
    <Cron
      value={expression}
      onChange={setExpression}
      onValidate={handleValidation}
      error={!isValid ? 'Invalid expression' : undefined}
    />
  );
}
```

## TypeScript Support

All components include comprehensive TypeScript definitions:

```tsx
import type {
  CronProps,
  CronExpression,
  CronValidationResult,
  CronPeriod
} from '@/components/cron';
```

## Examples

See `CronExample` component for comprehensive usage examples and interactive demos.

## Browser Support

- Modern browsers with ES2015+ support
- React 16.8+ (hooks support required)
- TypeScript 4.0+ recommended
