import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { AppCopy } from "../app/i18n";
import type { LayoutDirection } from "../app/types";

interface LayoutCanvasProps {
  copy: AppCopy;
  localName: string;
  localOsLabel: string;
  peerName: string;
  peerOsLabel: string;
  direction: LayoutDirection;
  onDirectionChange: (direction: LayoutDirection) => void;
}

interface ScreenBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

const SCREEN_W = 92;
const SCREEN_H = 56;

function computeBoxes(direction: LayoutDirection, canvasW: number, canvasH: number) {
  const cx = canvasW / 2;
  const cy = canvasH / 2;
  const gap = 14;

  const local: ScreenBox = {
    x: cx - SCREEN_W / 2,
    y: cy - SCREEN_H / 2,
    width: SCREEN_W,
    height: SCREEN_H
  };

  let peer: ScreenBox;
  switch (direction) {
    case "left":
      peer = { x: local.x - SCREEN_W - gap, y: local.y, width: SCREEN_W, height: SCREEN_H };
      break;
    case "right":
      peer = { x: local.x + SCREEN_W + gap, y: local.y, width: SCREEN_W, height: SCREEN_H };
      break;
    case "top":
      peer = { x: local.x, y: local.y - SCREEN_H - gap, width: SCREEN_W, height: SCREEN_H };
      break;
    case "bottom":
    default:
      peer = { x: local.x, y: local.y + SCREEN_H + gap, width: SCREEN_W, height: SCREEN_H };
  }

  return { local, peer };
}

function clampBox(box: ScreenBox, canvasW: number, canvasH: number): ScreenBox {
  return {
    width: box.width,
    height: box.height,
    x: Math.max(0, Math.min(box.x, canvasW - box.width)),
    y: Math.max(0, Math.min(box.y, canvasH - box.height))
  };
}

function deriveDirection(local: ScreenBox, peer: ScreenBox): LayoutDirection {
  const localCenterX = local.x + local.width / 2;
  const localCenterY = local.y + local.height / 2;
  const peerCenterX = peer.x + peer.width / 2;
  const peerCenterY = peer.y + peer.height / 2;
  const dx = peerCenterX - localCenterX;
  const dy = peerCenterY - localCenterY;

  if (Math.abs(dx) >= Math.abs(dy)) {
    return dx >= 0 ? "right" : "left";
  }
  return dy >= 0 ? "bottom" : "top";
}

export function LayoutCanvas({
  copy,
  localName,
  localOsLabel,
  peerName,
  peerOsLabel,
  direction,
  onDirectionChange
}: LayoutCanvasProps) {
  const canvasRef = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({ width: 560, height: 220 });
  const [peerBox, setPeerBox] = useState<ScreenBox>(() => computeBoxes(direction, 560, 220).peer);
  const [dragging, setDragging] = useState(false);
  const dragOffsetRef = useRef({ x: 0, y: 0 });
  const lastDirectionRef = useRef<LayoutDirection>(direction);

  const localBox = useMemo(
    () => computeBoxes(direction, size.width, size.height).local,
    [direction, size.width, size.height]
  );

  // Resize observer keeps the boxes centered when the panel changes width.
  useEffect(() => {
    const el = canvasRef.current;
    if (!el) {
      return;
    }
    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (!entry) return;
      const { width, height } = entry.contentRect;
      setSize({ width, height });
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  // When the upstream direction prop changes and we're not actively dragging,
  // snap the peer box to the canonical position for that direction.
  useEffect(() => {
    if (dragging) return;
    if (lastDirectionRef.current === direction) {
      // Recompute on size change.
      const { peer } = computeBoxes(direction, size.width, size.height);
      setPeerBox(peer);
      return;
    }
    lastDirectionRef.current = direction;
    const { peer } = computeBoxes(direction, size.width, size.height);
    setPeerBox(peer);
  }, [direction, size.width, size.height, dragging]);

  const onPointerDown = useCallback(
    (e: React.PointerEvent<HTMLDivElement>) => {
      e.preventDefault();
      const target = e.currentTarget;
      target.setPointerCapture(e.pointerId);
      const rect = canvasRef.current?.getBoundingClientRect();
      if (!rect) return;
      dragOffsetRef.current = {
        x: e.clientX - rect.left - peerBox.x,
        y: e.clientY - rect.top - peerBox.y
      };
      setDragging(true);
    },
    [peerBox.x, peerBox.y]
  );

  const onPointerMove = useCallback(
    (e: React.PointerEvent<HTMLDivElement>) => {
      if (!dragging) return;
      const rect = canvasRef.current?.getBoundingClientRect();
      if (!rect) return;
      const nextX = e.clientX - rect.left - dragOffsetRef.current.x;
      const nextY = e.clientY - rect.top - dragOffsetRef.current.y;
      const next = clampBox(
        { x: nextX, y: nextY, width: SCREEN_W, height: SCREEN_H },
        size.width,
        size.height
      );
      setPeerBox(next);

      // Live preview of which direction will be picked when released.
      const live = deriveDirection(localBox, next);
      if (live !== direction) {
        onDirectionChange(live);
        lastDirectionRef.current = live;
      }
    },
    [dragging, size.width, size.height, localBox, direction, onDirectionChange]
  );

  const onPointerUp = useCallback(
    (e: React.PointerEvent<HTMLDivElement>) => {
      if (!dragging) return;
      e.currentTarget.releasePointerCapture(e.pointerId);
      const finalDirection = deriveDirection(localBox, peerBox);
      const { peer } = computeBoxes(finalDirection, size.width, size.height);
      setPeerBox(peer);
      lastDirectionRef.current = finalDirection;
      onDirectionChange(finalDirection);
      setDragging(false);
    },
    [dragging, localBox, peerBox, size.width, size.height, onDirectionChange]
  );

  // Edge highlight rectangles between the two screens.
  const edgeStyle = useMemo(() => {
    switch (direction) {
      case "left":
        return {
          orientation: "vertical" as const,
          style: {
            left: `${localBox.x - 1}px`,
            top: `${localBox.y + 6}px`,
            height: `${localBox.height - 12}px`
          }
        };
      case "right":
        return {
          orientation: "vertical" as const,
          style: {
            left: `${localBox.x + localBox.width - 1}px`,
            top: `${localBox.y + 6}px`,
            height: `${localBox.height - 12}px`
          }
        };
      case "top":
        return {
          orientation: "horizontal" as const,
          style: {
            top: `${localBox.y - 1}px`,
            left: `${localBox.x + 6}px`,
            width: `${localBox.width - 12}px`
          }
        };
      case "bottom":
      default:
        return {
          orientation: "horizontal" as const,
          style: {
            top: `${localBox.y + localBox.height - 1}px`,
            left: `${localBox.x + 6}px`,
            width: `${localBox.width - 12}px`
          }
        };
    }
  }, [direction, localBox]);

  return (
    <div className="layout-board">
      <div className="layout-board-meta">
        <span>
          <strong>{copy.layout.title}</strong> · {copy.layout.canvasDirectionLabel(
            copy.states.layoutDirections[direction]
          )}
        </span>
        <span className="pill available">
          <span className="pill-dot" />
          {copy.states.layoutDirections[direction]}
        </span>
      </div>
      <div ref={canvasRef} className="layout-canvas">
        <div
          className={`layout-edge ${edgeStyle.orientation} is-active`}
          style={edgeStyle.style}
        />
        <div
          className="layout-screen local"
          style={{ left: localBox.x, top: localBox.y, width: SCREEN_W, height: SCREEN_H }}
          title={`${localName} · ${localOsLabel}`}
        >
          <span className="layout-screen-tag">{copy.layout.localTag}</span>
          <span className="layout-screen-name">{localName}</span>
        </div>
        <div
          className={dragging ? "layout-screen peer is-dragging" : "layout-screen peer"}
          style={{ left: peerBox.x, top: peerBox.y, width: SCREEN_W, height: SCREEN_H }}
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          onPointerCancel={onPointerUp}
          title={`${peerName} · ${peerOsLabel}`}
        >
          <span className="layout-screen-tag">{copy.layout.peerTag}</span>
          <span className="layout-screen-name">{peerName}</span>
        </div>
      </div>
      <p className="layout-hint">{copy.layout.dragHint}</p>
    </div>
  );
}
