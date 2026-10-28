import Header from './Header';
import Footer from './Footer';
import TabBar from './TabBar';

export default function SiteLayout({ children }) {
  return (
    <div className="site">
      <a href="#main" className="skip" onClick={(e) => { e.preventDefault(); document.getElementById('main')?.focus(); }}>
        Skip to content
      </a>
      <Header />
      <main id="main" tabIndex={-1} className="site__main">
        {children}
      </main>
      <Footer />
      <TabBar />
    </div>
  );
}
