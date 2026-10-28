import { useEffect, useRef } from 'react';
import { startLive } from '../lib/live';

/**
 * The animated backdrop (colour orbs + spotlight + grain) and the layer that click effects are drawn on.
 * Mounted once, behind and above everything else; it never receives pointer events.
 */
export default function LiveFX() {
  const aurora = useRef(null);
  const layer = useRef(null);

  useEffect(() => startLive({ aurora: aurora.current, layer: layer.current }), []);

  return (
    <>
      <div className="aurora" ref={aurora} aria-hidden="true">
        <div className="aurora__orb aurora__orb--a" style={{ '--d': 0.9 }}><i /></div>
        <div className="aurora__orb aurora__orb--b" style={{ '--d': -0.6 }}><i /></div>
        <div className="aurora__orb aurora__orb--c" style={{ '--d': 0.5 }}><i /></div>
        <div className="aurora__orb aurora__orb--d" style={{ '--d': -0.8 }}><i /></div>
        <div className="aurora__cursor" />
      </div>
      <div className="fx-layer" ref={layer} aria-hidden="true" />
    </>
  );
}
