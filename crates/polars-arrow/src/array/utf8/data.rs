use arrow_data::{ArrayData, ArrayDataBuilder};

use crate::array::{Arrow2Arrow, Utf8Array};
use crate::bitmap::Bitmap;
use crate::offset::{Offset, OffsetsBuffer};

impl<O: Offset> Arrow2Arrow for Utf8Array<O> {
    fn to_data(&self) -> ArrayData {
        let dtype = self.dtype().clone().into();
        let builder = ArrayDataBuilder::new(dtype)
            .len(self.offsets().len_proxy())
            .buffers(vec![
                self.offsets.clone().into_inner().into(),
                self.values.clone().into(),
            ])
            .nulls(self.validity.as_ref().map(|b| b.clone().into()));

        // SAFETY: Array is valid
        unsafe { builder.build_unchecked() }
    }

    fn from_data(data: &ArrayData) -> Self {
        let dtype = data.data_type().clone().into();
        if data.is_empty() {
            // Handle empty offsets
            return Self::new_empty(dtype);
        }

        let buffers = data.buffers();

        // SAFETY: ArrayData is valid
        let mut offsets = unsafe { OffsetsBuffer::new_unchecked(buffers[0].clone().into()) };
        offsets.slice(data.offset(), data.len() + 1);

        Self {
            dtype,
            offsets,
            values: buffers[1].clone().into(),
            validity: data.nulls().map(|n| Bitmap::from_null_buffer(n.clone())),
        }
    }
}
