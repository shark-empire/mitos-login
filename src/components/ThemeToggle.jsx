import { LuMoon, LuSun } from 'react-icons/lu';
import { useTheme } from '../context/Theme';

export function ThemeToggle() {
  const { theme, toggle } = useTheme();
  const Icon = theme === 'dark' ? LuSun : LuMoon;
  return (
    <button type="button" className="iconbtn" onClick={toggle} aria-label={theme === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'}>
      <Icon aria-hidden="true" />
    </button>
  );
}
