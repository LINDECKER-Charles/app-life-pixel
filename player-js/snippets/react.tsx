import '@life-pixel/player';
import type { LifePixelElement } from '@life-pixel/player';
import type { DetailedHTMLProps, HTMLAttributes } from 'react';

// Lets TypeScript check <life-pixel> in JSX; once per project.
declare module 'react' {
  namespace JSX {
    interface IntrinsicElements {
      'life-pixel': DetailedHTMLProps<HTMLAttributes<LifePixelElement>, LifePixelElement> & {
        src?: string;
        tag?: string;
        alt?: string;
        autoplay?: 'true' | 'false';
        loop?: boolean | 'true' | 'false';
        motion?: 'auto' | 'always';
      };
    }
  }
}

export function {{className}}() {
  return <life-pixel src="{{src}}"{{tagAttribute}} alt={{{altExpression}}}></life-pixel>;
}
