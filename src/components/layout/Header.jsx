import { useEffect, useState } from 'react';
import { LuMenu, LuX } from 'react-icons/lu';
import { Link, NavLink, useLocation } from '../../router';
import { useAuth } from '../../context/Auth';
import { ThemeToggle } from '../ThemeToggle';
import Logo from '../Logo';
import { Button } from '../ui';
import { MAIN_NAV } from './nav';

export default function Header() {
  const { user } = useAuth();
  const { path } = useLocation();
  const [open, setOpen] = useState(false);

  useEffect(() => setOpen(false), [path]);

  return (
    <header className="header">
      <div className="container header__bar">
        <Link to="/" className="header__brand" aria-label="SCI-FI DATA home">
          <Logo />
        </Link>

        <nav className={`header__nav${MAIN_NAV.length > 5 ? ' header__nav--many' : ''}`} aria-label="Main">
          {MAIN_NAV.map((n) => (
            <NavLink key={n.to} to={n.to} className="header__link">
              {n.label}
            </NavLink>
          ))}
        </nav>

        <div className="header__actions">
          <ThemeToggle />
          {user ? (
            <Button to="/account" variant="secondary" size="sm" className="header__auth">
              {user.first_name || 'Account'}
            </Button>
          ) : (
            <>
              <Button to="/login" variant="ghost" size="sm" className="header__auth">
                Log in
              </Button>
              <Button to="/signup" size="sm" className="header__auth">
                Sign up
              </Button>
            </>
          )}
          <button
            type="button"
            className="iconbtn header__menu"
            onClick={() => setOpen((o) => !o)}
            aria-expanded={open}
            aria-controls="mobile-menu"
            aria-label={open ? 'Close menu' : 'Open menu'}
          >
            {open ? <LuX aria-hidden="true" /> : <LuMenu aria-hidden="true" />}
          </button>
        </div>
      </div>

      {open && (
        <nav id="mobile-menu" className="mobilemenu" aria-label="Menu">
          <div className="container mobilemenu__list">
            {MAIN_NAV.map((n) => (
              <NavLink key={n.to} to={n.to} className="mobilemenu__link">
                {n.label}
              </NavLink>
            ))}
            <NavLink to="/support" className="mobilemenu__link">
              Support
            </NavLink>
            {user ? (
              <NavLink to="/account" className="mobilemenu__link">
                My account
              </NavLink>
            ) : (
              <div className="mobilemenu__auth">
                <Button to="/login" variant="secondary" block>
                  Log in
                </Button>
                <Button to="/signup" block>
                  Sign up
                </Button>
              </div>
            )}
          </div>
        </nav>
      )}
    </header>
  );
}
