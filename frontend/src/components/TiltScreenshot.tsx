import { useRef, useState, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { useTheme } from '@/context/ThemeContext';

const EASE = 'cubic-bezier(0.22, 1, 0.36, 1)';
const DURATION = 320; // ms — feel free to tweak

const TiltZoomScreenshot: React.FC<{
  lightSrc: string;
  darkSrc: string;
  alt: string;
  className?: string; // wrapper width utilities from parent grid
}> = ({ lightSrc, darkSrc, alt, className }) => {
  const { isDarkMode } = useTheme();
  const src = isDarkMode ? darkSrc : lightSrc;

  // thumbnail tilt
  const cardRef = useRef<HTMLDivElement | null>(null);
  const imgThumbRef = useRef<HTMLImageElement | null>(null);
  const [tiltStyle, setTiltStyle] = useState<React.CSSProperties>({
    transform: 'translateZ(0) rotateX(0deg) rotateY(0deg) scale(1)',
    transition: `transform 160ms ${EASE}`,
  });

  const onMove: React.MouseEventHandler<HTMLDivElement> = (e) => {
    const el = cardRef.current;
    if (!el) return;
    const r = el.getBoundingClientRect();
    const px = (e.clientX - r.left) / r.width;
    const py = (e.clientY - r.top) / r.height;
    const rotX = (py - 0.5) * 14;
    const rotY = (0.5 - px) * 14;
    setTiltStyle({
      transform: `translateZ(-22px) rotateX(${rotX}deg) rotateY(${rotY}deg) scale(1.07)`,
      transition: 'transform 70ms linear',
    });
  };
  const onLeave: React.MouseEventHandler<HTMLDivElement> = () =>
    setTiltStyle({
      transform: 'translateZ(0) rotateX(0deg) rotateY(0deg) scale(1)',
      transition: `transform 200ms ${EASE}`,
    });

  // lightbox / FLIP
  const [open, setOpen] = useState(false);
  const overlayImgRef = useRef<HTMLImageElement | null>(null);
  const backdropRef = useRef<HTMLDivElement | null>(null);
  const startRectRef = useRef<DOMRect | null>(null);

  // ESC closes
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => e.key === 'Escape' && close();
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open]);

  const openLightbox = () => {
    if (!imgThumbRef.current) return;
    startRectRef.current = imgThumbRef.current.getBoundingClientRect();
    setOpen(true);
    // after portal mounts, run FLIP
    requestAnimationFrame(() => {
      const img = overlayImgRef.current;
      const bd = backdropRef.current;
      if (!img || !bd || !startRectRef.current) return;

      const end = img.getBoundingClientRect();
      const s = startRectRef.current;
      const dx = s.left - end.left;
      const dy = s.top - end.top;
      const sx = s.width / end.width;
      const sy = s.height / end.height;

      img.style.transition = 'none';
      img.style.transformOrigin = 'top left';
      img.style.transform = `translate(${dx}px, ${dy}px) scale(${sx}, ${sy})`;
      bd.style.transition = 'none';
      bd.style.opacity = '0';

      // play
      // force reflow
      void img.offsetHeight;
      img.style.transition = `transform ${DURATION}ms ${EASE}`;
      bd.style.transition = `opacity ${DURATION}ms ${EASE}`;
      img.style.transform = 'translate(0, 0) scale(1, 1)';
      bd.style.opacity = '1';
    });
  };

  const close = () => {
    const img = overlayImgRef.current;
    const bd = backdropRef.current;
    const end = img?.getBoundingClientRect();
    const s = startRectRef.current;
    if (!img || !bd || !end || !s) {
      setOpen(false);
      return;
    }
    const dx = s.left - end.left;
    const dy = s.top - end.top;
    const sx = s.width / end.width;
    const sy = s.height / end.height;

    img.style.transformOrigin = 'top left';
    img.style.transform = `translate(${dx}px, ${dy}px) scale(${sx}, ${sy})`;
    bd.style.opacity = '0';

    window.setTimeout(() => setOpen(false), DURATION);
  };

  return (
    <>
      {/* Thumbnail / card */}
      <div
        className={className}
        style={{ perspective: '1200px' }}
        onMouseMove={onMove}
        onMouseLeave={onLeave}
        onClick={openLightbox}
      >
        <div
          ref={cardRef}
          className="
            rounded-2xl overflow-hidden cursor-zoom-in
            shadow-[0_15px_55px_-10px_rgba(0,0,0,0.35)]
            dark:shadow-[0_15px_55px_-10px_rgba(0,0,0,0.6)]
            transform-gpu will-change-transform
          "
          style={{ transformStyle: 'preserve-3d', ...tiltStyle }}
        >
          <img
            ref={imgThumbRef}
            src={src}
            alt={alt}
            className="block w-full h-auto select-none"
            draggable={false}
          />
        </div>
      </div>

      {/* Lightbox portal (animate from thumb → center and back) */}
      {open &&
        createPortal(
          <div
            ref={backdropRef}
            className="
              fixed inset-0 z-[100] flex items-center justify-center p-4 sm:p-6
              bg-black/60 backdrop-blur-[2px]
            "
            onClick={close}
            role="dialog"
            aria-modal="true"
          >
            <div
              className="relative max-w-[95vw] max-h-[90vh]"
              onClick={(e) => e.stopPropagation()}
            >
              <img
                ref={overlayImgRef}
                src={src}
                alt={alt}
                className="
                  block max-w-[95vw] max-h-[90vh] w-auto h-auto rounded-2xl
                  shadow-[0_25px_85px_-20px_rgba(0,0,0,0.55)]
                  dark:shadow-[0_25px_85px_-20px_rgba(0,0,0,0.75)]
                  transform-gpu will-change-transform select-none
                "
                draggable={false}
                style={{ transformOrigin: 'top left' }}
              />
              <button
                onClick={close}
                className="
                  absolute -top-3 -right-3 sm:-top-4 sm:-right-4
                  h-10 w-10 rounded-full grid place-items-center
                  bg-white/95 dark:bg-gray-900/90 text-gray-700 dark:text-gray-200
                  shadow-lg hover:scale-105 active:scale-95 transition
                "
                aria-label="Close"
              >
                ✕
              </button>
            </div>
          </div>,
          document.body,
        )}
    </>
  );
};

export default TiltZoomScreenshot;
