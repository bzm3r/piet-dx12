// Copyright © 2019 piet-dx12 developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate winapi;
extern crate wio;

use self::winapi::um::d3dcommon::ID3DBlob;
use crate::error;
use crate::error::error_if_failed_else_unit;
use std::convert::TryFrom;
use std::{ffi, mem, path::Path, ptr};
use winapi::shared::{
    dxgi, dxgi1_2, dxgi1_3, dxgi1_4, dxgiformat, dxgitype, minwindef, windef, winerror,
};
use winapi::um::{d3d12, d3d12sdklayers, d3dcommon, d3dcompiler, dxgidebug, synchapi, winnt};
use winapi::Interface;
use wio::com::ComPtr;

// everything is ripped from d3d12-rs, but wio::com::ComPtr, and winapi are used more directly

#[derive(Clone)]
pub struct Heap(pub ComPtr<d3d12::ID3D12Heap>);

#[derive(Clone)]
pub struct Resource {
    pub com_ptr: ComPtr<d3d12::ID3D12Resource>,
    pub descriptor_heap_offset: u32,
}

pub struct VertexBufferView(pub ComPtr<d3d12::D3D12_VERTEX_BUFFER_VIEW>);

#[derive(Clone)]
pub struct Adapter1(pub ComPtr<dxgi::IDXGIAdapter1>);
#[derive(Clone)]
pub struct Factory2(pub ComPtr<dxgi1_2::IDXGIFactory2>);
#[derive(Clone)]
pub struct Factory4(pub ComPtr<dxgi1_4::IDXGIFactory4>);
#[derive(Clone)]
pub struct SwapChain(pub ComPtr<dxgi::IDXGISwapChain>);
#[derive(Clone)]
pub struct SwapChain1(pub ComPtr<dxgi1_2::IDXGISwapChain1>);
#[derive(Clone)]
pub struct SwapChain3(pub ComPtr<dxgi1_4::IDXGISwapChain3>);

#[derive(Clone)]
pub struct Device(pub ComPtr<d3d12::ID3D12Device>);

#[derive(Clone)]
pub struct CommandQueue(pub ComPtr<d3d12::ID3D12CommandQueue>);

#[derive(Clone)]
pub struct CommandAllocator(pub ComPtr<d3d12::ID3D12CommandAllocator>);

pub type CpuDescriptor = d3d12::D3D12_CPU_DESCRIPTOR_HANDLE;
pub type GpuDescriptor = d3d12::D3D12_GPU_DESCRIPTOR_HANDLE;

#[derive(Clone)]
pub struct DescriptorHeap {
    pub heap_type: d3d12::D3D12_DESCRIPTOR_HEAP_TYPE,
    pub increment_size: u32,
    pub heap: ComPtr<d3d12::ID3D12DescriptorHeap>,
}

pub type TextureAddressMode = [d3d12::D3D12_TEXTURE_ADDRESS_MODE; 3];

#[derive(Clone)]
pub struct RootSignature(pub ComPtr<d3d12::ID3D12RootSignature>);

#[derive(Clone)]
pub struct CommandSignature(pub ComPtr<d3d12::ID3D12CommandSignature>);
#[derive(Clone)]
pub struct GraphicsCommandList(pub ComPtr<d3d12::ID3D12GraphicsCommandList>);

#[derive(Clone)]
pub struct Event(pub winnt::HANDLE);
#[derive(Clone)]
pub struct Fence(pub ComPtr<d3d12::ID3D12Fence>);

#[derive(Clone)]
pub struct PipelineState(pub ComPtr<d3d12::ID3D12PipelineState>);

#[derive(Clone)]
pub struct CachedPSO(d3d12::D3D12_CACHED_PIPELINE_STATE);

#[derive(Clone)]
pub struct Blob(pub ComPtr<d3dcommon::ID3DBlob>);

#[derive(Clone)]
pub struct ShaderByteCode {
    pub bytecode: d3d12::D3D12_SHADER_BYTECODE,
    blob: Option<Blob>,
}

pub struct DebugController(pub d3d12sdklayers::ID3D12Debug);

#[derive(Clone)]
pub struct QueryHeap(pub ComPtr<d3d12::ID3D12QueryHeap>);

impl Resource {
    pub unsafe fn new(
        com_ptr: ComPtr<d3d12::ID3D12Resource>,
        descriptor_heap_offset: u32,
    ) -> Resource {
        Resource {
            com_ptr,
            descriptor_heap_offset,
        }
    }

    pub unsafe fn upload_data_to_resource<T>(&self, count: usize, data: *const T) {
        let mut mapped_memory = ptr::null_mut();
        let zero_range = d3d12::D3D12_RANGE { ..mem::zeroed() };
        error::error_if_failed_else_unit(self.com_ptr.Map(
            0,
            &zero_range as *const _,
            &mut mapped_memory as *mut _ as *mut _,
        ))
        .expect("could not map GPU mem to CPU mem");

        ptr::copy(data, mapped_memory, count);
        self.com_ptr.Unmap(0, ptr::null());
    }

    pub unsafe fn download_data_from_resource<T>(&self, count: usize) -> Vec<T> {
        let data_size_in_bytes = mem::size_of::<T>();
        let mut mapped_memory = ptr::null_mut();
        let zero_range = d3d12::D3D12_RANGE {
            Begin: 0,
            End: data_size_in_bytes * count,
        };
        error::error_if_failed_else_unit(self.com_ptr.Map(
            0,
            &zero_range as *const _,
            &mut mapped_memory as *mut _ as *mut _,
        ))
        .expect("could not map GPU mem to CPU mem");
        let mut result: Vec<T> = Vec::new();
        result.reserve(count);
        ptr::copy(
            mapped_memory as *const T,
            result.as_mut_ptr() as *mut T,
            count,
        );
        result.set_len(count);
        self.com_ptr.Unmap(0, ptr::null());

        result
    }

    pub unsafe fn get_gpu_virtual_address(&self) -> d3d12::D3D12_GPU_VIRTUAL_ADDRESS {
        self.com_ptr.GetGPUVirtualAddress()
    }
}

impl Factory4 {
    pub unsafe fn create(flags: minwindef::UINT) -> Factory4 {
        let mut factory = ptr::null_mut();

        error::error_if_failed_else_unit(dxgi1_3::CreateDXGIFactory2(
            flags,
            &dxgi1_4::IDXGIFactory4::uuidof(),
            &mut factory as *mut _ as *mut _,
        ))
        .expect("could not create factory4");

        Factory4(ComPtr::from_raw(factory))
    }

    pub unsafe fn enumerate_adapters(
        &self,
        id: u32,
    ) -> (*mut dxgi::IDXGIAdapter1, winerror::HRESULT) {
        let mut adapter = ptr::null_mut();
        let hr = self.0.EnumAdapters1(id, &mut adapter as *mut _ as *mut _);

        (adapter, hr)
    }

    pub unsafe fn create_swapchain_for_hwnd(
        &self,
        command_queue: CommandQueue,
        hwnd: windef::HWND,
        desc: dxgi1_2::DXGI_SWAP_CHAIN_DESC1,
    ) -> SwapChain3 {
        let mut swap_chain = ptr::null_mut();
        error::error_if_failed_else_unit(self.0.CreateSwapChainForHwnd(
            command_queue.0.as_raw() as *mut _,
            hwnd,
            &desc,
            ptr::null(),
            ptr::null_mut(),
            &mut swap_chain as *mut _ as *mut _,
        ))
        .expect("could not creation swapchain for hwnd");

        SwapChain3(ComPtr::from_raw(swap_chain))
    }
}

impl CommandQueue {
    pub unsafe fn signal(&self, fence: Fence, value: u64) -> winerror::HRESULT {
        self.0.Signal(fence.0.as_raw(), value)
    }

    pub unsafe fn execute_command_lists(
        &self,
        num_command_lists: u32,
        command_lists: &[*mut d3d12::ID3D12CommandList],
    ) {
        self.0
            .ExecuteCommandLists(num_command_lists, command_lists.as_ptr());
    }

    pub unsafe fn get_timestamp_frequency(&self) -> u64 {
        let mut result: u64 = 0;

        error_if_failed_else_unit(self.0.GetTimestampFrequency(&mut result as *mut _))
            .expect("could not get timestamp frequency");

        result
    }
}

impl SwapChain {
    pub unsafe fn get_buffer(&self, id: u32) -> Resource {
        let mut resource = ptr::null_mut();
        error::error_if_failed_else_unit(self.0.GetBuffer(
            id,
            &d3d12::ID3D12Resource::uuidof(),
            &mut resource as *mut _ as *mut _,
        ))
        .expect("SwapChain could not get buffer");

        Resource::new(ComPtr::from_raw(resource), 0)
    }

    // TODO: present flags
    pub unsafe fn present(&self, interval: u32, flags: u32) -> winerror::HRESULT {
        self.0.Present(interval, flags)
    }
}

impl SwapChain1 {
    pub unsafe fn cast_into_swap_chain3(&self) -> SwapChain3 {
        SwapChain3(
            self.0
                .cast::<dxgi1_4::IDXGISwapChain3>()
                .expect("could not cast into SwapChain3"),
        )
    }

    pub unsafe fn get_buffer(&self, id: u32) -> Resource {
        let mut resource = ptr::null_mut();
        error::error_if_failed_else_unit(self.0.GetBuffer(
            id,
            &d3d12::ID3D12Resource::uuidof(),
            &mut resource as *mut _ as *mut _,
        ))
        .expect("SwapChain1 could not get buffer");

        Resource::new(ComPtr::from_raw(resource), 0)
    }
}

impl SwapChain3 {
    pub unsafe fn get_buffer(&self, id: u32) -> Resource {
        let mut resource = ptr::null_mut();
        error::error_if_failed_else_unit(self.0.GetBuffer(
            id,
            &d3d12::ID3D12Resource::uuidof(),
            &mut resource as *mut _ as *mut _,
        ))
        .expect("SwapChain3 could not get buffer");

        Resource::new(ComPtr::from_raw(resource), 0)
    }

    pub unsafe fn get_current_back_buffer_index(&self) -> u32 {
        self.0.GetCurrentBackBufferIndex()
    }

    pub unsafe fn present(&self, interval: u32, flags: u32) -> Result<(), String> {
        error::error_if_failed_else_unit(self.0.Present1(
            interval,
            flags,
            &dxgi1_2::DXGI_PRESENT_PARAMETERS { ..mem::zeroed() } as *const _,
        ))
    }
}

impl Blob {
    pub unsafe fn print_to_console(blob: Blob) {
        println!("==SHADER COMPILE MESSAGES==");
        let message = {
            let pointer = blob.0.GetBufferPointer();
            let size = blob.0.GetBufferSize();
            let slice = std::slice::from_raw_parts(pointer as *const u8, size as usize);
            String::from_utf8_lossy(slice).into_owned()
        };
        println!("{}", message);
        println!("===========================");
    }
}

impl Device {
    pub unsafe fn create_device(factory4: &Factory4) -> Result<Device, Vec<winerror::HRESULT>> {
        let mut id = 0;
        let mut errors: Vec<winerror::HRESULT> = Vec::new();

        loop {
            let adapter = {
                let (adapter, hr) = factory4.enumerate_adapters(id);

                if !winerror::SUCCEEDED(hr) {
                    errors.push(hr);
                    return Err(errors);
                }

                ComPtr::from_raw(adapter)
            };

            id += 1;

            let (device, hr) =
                Device::create_using_adapter(adapter.clone(), d3dcommon::D3D_FEATURE_LEVEL_12_0);

            if !winerror::SUCCEEDED(hr) {
                errors.push(hr);
                continue;
            } else {
                std::mem::drop(adapter);
                return Ok(Device(ComPtr::from_raw(device)));
            }
        }
    }

    pub unsafe fn create_using_adapter<I: Interface>(
        adapter: ComPtr<I>,
        feature_level: d3dcommon::D3D_FEATURE_LEVEL,
    ) -> (*mut d3d12::ID3D12Device, winerror::HRESULT) {
        let mut device = ptr::null_mut();
        let hr = d3d12::D3D12CreateDevice(
            adapter.as_raw() as *mut _,
            feature_level as _,
            &d3d12::ID3D12Device::uuidof(),
            &mut device as *mut _ as *mut _,
        );

        (device, hr)
    }

    pub unsafe fn create_command_allocator(
        &self,
        list_type: d3d12::D3D12_COMMAND_LIST_TYPE,
    ) -> CommandAllocator {
        let mut allocator = ptr::null_mut();
        error::error_if_failed_else_unit(self.0.CreateCommandAllocator(
            list_type,
            &d3d12::ID3D12CommandAllocator::uuidof(),
            &mut allocator as *mut _ as *mut _,
        ))
        .expect("device could nto create command allocator");

        CommandAllocator(ComPtr::from_raw(allocator))
    }

    pub unsafe fn create_command_queue(
        &self,
        list_type: d3d12::D3D12_COMMAND_LIST_TYPE,
        priority: minwindef::INT,
        flags: d3d12::D3D12_COMMAND_QUEUE_FLAGS,
        node_mask: minwindef::UINT,
    ) -> CommandQueue {
        let desc = d3d12::D3D12_COMMAND_QUEUE_DESC {
            Type: list_type,
            Priority: priority,
            Flags: flags,
            NodeMask: node_mask,
        };

        let mut cmd_q = ptr::null_mut();
        error::error_if_failed_else_unit(self.0.CreateCommandQueue(
            &desc,
            &d3d12::ID3D12CommandQueue::uuidof(),
            &mut cmd_q as *mut _ as *mut _,
        ))
        .expect("device could not create command queue");

        CommandQueue(ComPtr::from_raw(cmd_q))
    }

    pub unsafe fn create_descriptor_heap(
        &self,
        heap_description: &d3d12::D3D12_DESCRIPTOR_HEAP_DESC,
    ) -> DescriptorHeap {
        let mut heap = ptr::null_mut();
        error::error_if_failed_else_unit(self.0.CreateDescriptorHeap(
            heap_description,
            &d3d12::ID3D12DescriptorHeap::uuidof(),
            &mut heap as *mut _ as *mut _,
        ))
        .expect("device could not create descriptor heap");

        DescriptorHeap {
            heap_type: heap_description.Type,
            increment_size: self.get_descriptor_increment_size(heap_description.Type),
            heap: ComPtr::from_raw(heap),
        }
    }

    pub unsafe fn get_descriptor_increment_size(
        &self,
        heap_type: d3d12::D3D12_DESCRIPTOR_HEAP_TYPE,
    ) -> u32 {
        self.0.GetDescriptorHandleIncrementSize(heap_type)
    }

    pub unsafe fn create_graphics_pipeline_state(
        &self,
        graphics_pipeline_desc: &d3d12::D3D12_GRAPHICS_PIPELINE_STATE_DESC,
    ) -> PipelineState {
        let mut pipeline_state = ptr::null_mut();

        error::error_if_failed_else_unit(self.0.CreateGraphicsPipelineState(
            graphics_pipeline_desc as *const _,
            &d3d12::ID3D12PipelineState::uuidof(),
            &mut pipeline_state as *mut _ as *mut _,
        ))
        .expect("device could not create graphics pipeline state");

        PipelineState(ComPtr::from_raw(pipeline_state))
    }

    pub unsafe fn create_compute_pipeline_state(
        &self,
        compute_pipeline_desc: &d3d12::D3D12_COMPUTE_PIPELINE_STATE_DESC,
    ) -> PipelineState {
        let mut pipeline_state = ptr::null_mut();

        error::error_if_failed_else_unit(self.0.CreateComputePipelineState(
            compute_pipeline_desc as *const _,
            &d3d12::ID3D12PipelineState::uuidof(),
            &mut pipeline_state as *mut _ as *mut _,
        ))
        .expect("device could not create compute pipeline state");

        PipelineState(ComPtr::from_raw(pipeline_state))
    }

    pub unsafe fn create_root_signature(
        &self,
        node_mask: minwindef::UINT,
        blob: Blob,
    ) -> RootSignature {
        let mut signature = ptr::null_mut();
        error::error_if_failed_else_unit(self.0.CreateRootSignature(
            node_mask,
            blob.0.GetBufferPointer(),
            blob.0.GetBufferSize(),
            &d3d12::ID3D12RootSignature::uuidof(),
            &mut signature as *mut _ as *mut _,
        ))
        .expect("device could not create root signature");

        RootSignature(ComPtr::from_raw(signature))
    }

    pub unsafe fn create_command_signature(
        &self,
        root_signature: RootSignature,
        arguments: &[d3d12::D3D12_INDIRECT_ARGUMENT_DESC],
        stride: u32,
        node_mask: minwindef::UINT,
    ) -> CommandSignature {
        let mut signature = ptr::null_mut();
        let desc = d3d12::D3D12_COMMAND_SIGNATURE_DESC {
            ByteStride: stride,
            NumArgumentDescs: arguments.len() as _,
            pArgumentDescs: arguments.as_ptr() as *const _,
            NodeMask: node_mask,
        };

        error::error_if_failed_else_unit(self.0.CreateCommandSignature(
            &desc,
            root_signature.0.as_raw(),
            &d3d12::ID3D12CommandSignature::uuidof(),
            &mut signature as *mut _ as *mut _,
        ))
        .expect("device could not create command signature");

        CommandSignature(ComPtr::from_raw(signature))
    }

    pub unsafe fn create_graphics_command_list(
        &self,
        list_type: d3d12::D3D12_COMMAND_LIST_TYPE,
        allocator: CommandAllocator,
        initial_ps: PipelineState,
        node_mask: minwindef::UINT,
    ) -> GraphicsCommandList {
        let mut command_list = ptr::null_mut();

        error::error_if_failed_else_unit(self.0.CreateCommandList(
            node_mask,
            list_type,
            allocator.0.as_raw(),
            initial_ps.0.as_raw(),
            &d3d12::ID3D12GraphicsCommandList::uuidof(),
            &mut command_list as *mut _ as *mut _,
        ))
        .expect("device could not create graphics command list");

        GraphicsCommandList(ComPtr::from_raw(command_list))
    }

    pub unsafe fn create_byte_addressed_buffer_unordered_access_view(
        &self,
        resource: Resource,
        descriptor: CpuDescriptor,
        first_element: u64,
        num_elements: u32,
    ) {
        // shouldn't flags be dxgiformat::DXGI_FORMAT_R32_TYPELESS?
        let mut uav_desc = d3d12::D3D12_UNORDERED_ACCESS_VIEW_DESC {
            Format: dxgiformat::DXGI_FORMAT_R32_TYPELESS,
            ViewDimension: d3d12::D3D12_UAV_DIMENSION_BUFFER,
            ..mem::zeroed()
        };
        *uav_desc.u.Buffer_mut() = d3d12::D3D12_BUFFER_UAV {
            FirstElement: first_element,
            NumElements: num_elements,
            // shouldn't StructureByteStride be 0?
            StructureByteStride: 0,
            CounterOffsetInBytes: 0,
            // shouldn't flags be d3d12::D3D12_BUFFER_UAV_FLAG_RAW?
            Flags: d3d12::D3D12_BUFFER_UAV_FLAG_RAW,
        };
        self.0.CreateUnorderedAccessView(
            resource.com_ptr.as_raw(),
            ptr::null_mut(),
            &uav_desc as *const _,
            descriptor,
        )
    }

    pub unsafe fn create_unordered_access_view(
        &self,
        resource: Resource,
        descriptor: CpuDescriptor,
    ) {
        self.0.CreateUnorderedAccessView(
            resource.com_ptr.as_raw(),
            ptr::null_mut(),
            ptr::null(),
            descriptor,
        )
    }

    pub unsafe fn create_constant_buffer_view(
        &self,
        resource: Resource,
        descriptor: CpuDescriptor,
        size_in_bytes: u32,
    ) {
        let cbv_desc = d3d12::D3D12_CONSTANT_BUFFER_VIEW_DESC {
            BufferLocation: resource.get_gpu_virtual_address(),
            SizeInBytes: size_in_bytes,
        };
        self.0
            .CreateConstantBufferView(&cbv_desc as *const _, descriptor);
    }

    pub unsafe fn create_byte_addressed_buffer_shader_resource_view(
        &self,
        resource: Resource,
        descriptor: CpuDescriptor,
        first_element: u64,
        num_elements: u32,
    ) {
        let mut srv_desc = d3d12::D3D12_SHADER_RESOURCE_VIEW_DESC {
            // shouldn't flags be dxgiformat::DXGI_FORMAT_R32_TYPELESS?
            Format: dxgiformat::DXGI_FORMAT_R32_TYPELESS,
            ViewDimension: d3d12::D3D12_SRV_DIMENSION_BUFFER,
            Shader4ComponentMapping: 0x1688,
            ..mem::zeroed()
        };
        *srv_desc.u.Buffer_mut() = d3d12::D3D12_BUFFER_SRV {
            FirstElement: first_element,
            NumElements: num_elements,
            // shouldn't StructureByteStride be 0?
            StructureByteStride: 0,
            // shouldn't flags be d3d12::D3D12_BUFFER_SRV_FLAG_RAW?
            Flags: d3d12::D3D12_BUFFER_SRV_FLAG_RAW,
        };
        self.0.CreateShaderResourceView(
            resource.com_ptr.as_raw(),
            &srv_desc as *const _,
            descriptor,
        );
    }

    pub unsafe fn create_structured_buffer_shader_resource_view(
        &self,
        resource: Resource,
        descriptor: CpuDescriptor,
        first_element: u64,
        num_elements: u32,
        structure_byte_stride: u32,
    ) {
        let mut srv_desc = d3d12::D3D12_SHADER_RESOURCE_VIEW_DESC {
            Format: dxgiformat::DXGI_FORMAT_UNKNOWN,
            ViewDimension: d3d12::D3D12_SRV_DIMENSION_BUFFER,
            Shader4ComponentMapping: 0x1688,
            ..mem::zeroed()
        };
        *srv_desc.u.Buffer_mut() = d3d12::D3D12_BUFFER_SRV {
            FirstElement: first_element,
            NumElements: num_elements,
            StructureByteStride: structure_byte_stride,
            Flags: d3d12::D3D12_BUFFER_SRV_FLAG_NONE,
        };
        self.0.CreateShaderResourceView(
            resource.com_ptr.as_raw(),
            &srv_desc as *const _,
            descriptor,
        );
    }

    pub unsafe fn create_texture2d_shader_resource_view(
        &self,
        resource: Resource,
        format: dxgiformat::DXGI_FORMAT,
        descriptor: CpuDescriptor,
    ) {
        let mut srv_desc = d3d12::D3D12_SHADER_RESOURCE_VIEW_DESC {
            Format: format,
            ViewDimension: d3d12::D3D12_SRV_DIMENSION_TEXTURE2D,
            Shader4ComponentMapping: 0x1688,
            ..mem::zeroed()
        };
        *srv_desc.u.Texture2D_mut() = d3d12::D3D12_TEX2D_SRV {
            MostDetailedMip: 0,
            MipLevels: 1,
            PlaneSlice: 0,
            ResourceMinLODClamp: 0.0,
        };
        self.0.CreateShaderResourceView(
            resource.com_ptr.as_raw(),
            &srv_desc as *const _,
            descriptor,
        );
    }

    pub unsafe fn create_render_target_view(
        &self,
        resource: Resource,
        desc: *const d3d12::D3D12_RENDER_TARGET_VIEW_DESC,
        descriptor: CpuDescriptor,
    ) {
        self.0
            .CreateRenderTargetView(resource.com_ptr.as_raw(), desc, descriptor);
    }

    // TODO: interface not complete
    pub unsafe fn create_fence(&self, initial: u64) -> Fence {
        let mut fence = ptr::null_mut();
        error::error_if_failed_else_unit(self.0.CreateFence(
            initial,
            d3d12::D3D12_FENCE_FLAG_NONE,
            &d3d12::ID3D12Fence::uuidof(),
            &mut fence as *mut _ as *mut _,
        ))
        .expect("device could not create fence");

        Fence(ComPtr::from_raw(fence))
    }

    pub unsafe fn create_committed_resource(
        &self,
        heap_properties: &d3d12::D3D12_HEAP_PROPERTIES,
        flags: d3d12::D3D12_HEAP_FLAGS,
        resource_description: &d3d12::D3D12_RESOURCE_DESC,
        initial_resource_state: d3d12::D3D12_RESOURCE_STATES,
        optimized_clear_value: *const d3d12::D3D12_CLEAR_VALUE,
        descriptor_heap_offset: u32,
    ) -> Resource {
        let mut resource = ptr::null_mut();

        error::error_if_failed_else_unit(self.0.CreateCommittedResource(
            heap_properties as *const _,
            flags,
            resource_description as *const _,
            initial_resource_state,
            optimized_clear_value,
            &d3d12::ID3D12Resource::uuidof(),
            &mut resource as *mut _ as *mut _,
        ))
        .expect("device could not create committed resource");

        Resource::new(ComPtr::from_raw(resource), descriptor_heap_offset)
    }

    pub unsafe fn create_query_heap(
        &self,
        heap_type: d3d12::D3D12_QUERY_HEAP_TYPE,
        num_expected_queries: u32,
    ) -> QueryHeap {
        let query_heap_desc = d3d12::D3D12_QUERY_HEAP_DESC {
            Type: heap_type,
            Count: num_expected_queries,
            NodeMask: 0,
        };

        let mut query_heap = ptr::null_mut();

        error_if_failed_else_unit(self.0.CreateQueryHeap(
            &query_heap_desc as *const _,
            &d3d12::ID3D12QueryHeap::uuidof(),
            &mut query_heap as *mut _ as *mut _,
        ))
        .expect("could not create query heap");

        QueryHeap(ComPtr::from_raw(query_heap))
    }

    // based on: https://github.com/microsoft/DirectX-Graphics-Samples/blob/682051ddbe4be820195fffed0bfbdbbde8611a90/Libraries/D3DX12/d3dx12.h#L1875
    pub unsafe fn get_required_intermediate_buffer_size(
        &self,
        dest_resource: Resource,
        first_subresource: u32,
        num_subresources: u32,
    ) -> u64 {
        let desc: d3d12::D3D12_RESOURCE_DESC = dest_resource.com_ptr.GetDesc();

        let mut required_size: *mut u64 = ptr::null_mut();
        self.0.GetCopyableFootprints(
            &desc as *const _,
            first_subresource,
            num_subresources,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut required_size as *mut _ as *mut _,
        );

        *required_size
    }

    pub unsafe fn get_copyable_footprint(
        &self,
        first_subresource: u32,
        num_subresources: usize,
        base_offset: u64,
        dest_resource: Resource,
    ) -> (
        Vec<d3d12::D3D12_PLACED_SUBRESOURCE_FOOTPRINT>,
        Vec<u32>,
        Vec<u64>,
        u64,
    ) {
        let desc: d3d12::D3D12_RESOURCE_DESC = dest_resource.com_ptr.GetDesc();

        let mut layouts: Vec<d3d12::D3D12_PLACED_SUBRESOURCE_FOOTPRINT> =
            Vec::with_capacity(num_subresources);

        let mut num_rows: Vec<u32> = Vec::with_capacity(num_subresources);

        let mut row_size_in_bytes: Vec<u64> = Vec::with_capacity(num_subresources);

        let mut total_size: u64 = 0;

        self.0.GetCopyableFootprints(
            &desc as *const _,
            first_subresource,
            u32::try_from(num_subresources)
                .expect("could not safely convert num_subresources into u32"),
            base_offset,
            layouts.as_mut_ptr(),
            num_rows.as_mut_ptr(),
            row_size_in_bytes.as_mut_ptr(),
            &mut total_size as *mut _,
        );

        layouts.set_len(num_subresources);
        num_rows.set_len(num_subresources);
        row_size_in_bytes.set_len(num_subresources);

        (layouts, num_rows, row_size_in_bytes, total_size)
    }

    pub unsafe fn create_uploadable_buffer(
        &self,
        descriptor_heap_offset: u32,
        buffer_size_in_bytes: u64,
    ) -> Resource {
        let heap_properties = d3d12::D3D12_HEAP_PROPERTIES {
            //for GPU access only
            Type: d3d12::D3D12_HEAP_TYPE_UPLOAD,
            CPUPageProperty: d3d12::D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            //TODO: what should MemoryPoolPreference flag be?
            MemoryPoolPreference: d3d12::D3D12_MEMORY_POOL_UNKNOWN,
            //we don't care about multi-adapter operation, so these next two will be zero
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        };
        let resource_description = d3d12::D3D12_RESOURCE_DESC {
            Dimension: d3d12::D3D12_RESOURCE_DIMENSION_BUFFER,
            Width: buffer_size_in_bytes,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: d3d12::D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            Flags: d3d12::D3D12_RESOURCE_FLAG_NONE,
            ..mem::zeroed()
        };

        let buffer = self.create_committed_resource(
            &heap_properties,
            //TODO: is this heap flag ok?
            d3d12::D3D12_HEAP_FLAG_NONE,
            &resource_description,
            d3d12::D3D12_RESOURCE_STATE_GENERIC_READ,
            ptr::null(),
            descriptor_heap_offset,
        );

        buffer
    }

    pub unsafe fn create_uploadable_byte_addressed_buffer(
        &self,
        descriptor_heap_offset: u32,
        buffer_size_in_u32s: u32,
    ) -> Resource {
        let buffer_size_in_bytes = buffer_size_in_u32s as usize * mem::size_of::<u32>();

        let heap_properties = d3d12::D3D12_HEAP_PROPERTIES {
            Type: d3d12::D3D12_HEAP_TYPE_UPLOAD,
            CPUPageProperty: d3d12::D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            //TODO: what should MemoryPoolPreference flag be?
            MemoryPoolPreference: d3d12::D3D12_MEMORY_POOL_UNKNOWN,
            //we don't care about multi-adapter operation, so these next two will be zero
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        };
        let resource_description = d3d12::D3D12_RESOURCE_DESC {
            Dimension: d3d12::D3D12_RESOURCE_DIMENSION_BUFFER,
            Width: buffer_size_in_bytes as u64,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: d3d12::D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            Flags: d3d12::D3D12_RESOURCE_FLAG_NONE,
            ..mem::zeroed()
        };

        let byte_addressed_buffer = self.create_committed_resource(
            &heap_properties,
            //TODO: is this heap flag ok?
            d3d12::D3D12_HEAP_FLAG_NONE,
            &resource_description,
            d3d12::D3D12_RESOURCE_STATE_GENERIC_READ,
            ptr::null(),
            descriptor_heap_offset,
        );

        byte_addressed_buffer
    }

    pub unsafe fn create_gpu_only_byte_addressed_buffer(
        &self,
        descriptor_heap_offset: u32,
        buffer_size_in_u32s: u32,
    ) -> Resource {
        let size_of_u32_in_bytes = mem::size_of::<u32>();
        let buffer_size_in_bytes = buffer_size_in_u32s as usize * size_of_u32_in_bytes;

        //TODO: consider flag D3D12_HEAP_FLAG_ALLOW_SHADER_ATOMICS?
        let heap_properties = d3d12::D3D12_HEAP_PROPERTIES {
            //for GPU access only
            Type: d3d12::D3D12_HEAP_TYPE_DEFAULT,
            CPUPageProperty: d3d12::D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            //TODO: what should MemoryPoolPreference flag be?
            MemoryPoolPreference: d3d12::D3D12_MEMORY_POOL_UNKNOWN,
            //we don't care about multi-adapter operation, so these next two will be zero
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        };
        let resource_description = d3d12::D3D12_RESOURCE_DESC {
            Dimension: d3d12::D3D12_RESOURCE_DIMENSION_BUFFER,
            Width: buffer_size_in_bytes as u64,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            //essentially we're letting the adapter decide the layout
            Layout: d3d12::D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            Flags: d3d12::D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            ..mem::zeroed()
        };

        let buffer = self.create_committed_resource(
            &heap_properties,
            d3d12::D3D12_HEAP_FLAG_NONE,
            &resource_description,
            d3d12::D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            ptr::null(),
            descriptor_heap_offset,
        );

        buffer
    }

    pub unsafe fn create_gpu_only_texture2d_buffer(
        &self,
        descriptor_heap_offset: u32,
        width: u64,
        height: u32,
        format: dxgiformat::DXGI_FORMAT,
        allow_unordered_access: bool,
    ) -> Resource {
        let heap_properties = d3d12::D3D12_HEAP_PROPERTIES {
            Type: d3d12::D3D12_HEAP_TYPE_DEFAULT,
            CPUPageProperty: d3d12::D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            //TODO: what should MemoryPoolPreference flag be?
            MemoryPoolPreference: d3d12::D3D12_MEMORY_POOL_UNKNOWN,
            //we don't care about multi-adapter operation, so these next two will be zero
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        };

        let (flags, initial_resource_state) = {
            if allow_unordered_access {
                (
                    d3d12::D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
                    d3d12::D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                )
            } else {
                (
                    d3d12::D3D12_RESOURCE_FLAG_NONE,
                    d3d12::D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                )
            }
        };

        let resource_description = d3d12::D3D12_RESOURCE_DESC {
            Dimension: d3d12::D3D12_RESOURCE_DIMENSION_TEXTURE2D,
            Width: width,
            Height: height,
            DepthOrArraySize: 1,
            MipLevels: 1,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: d3d12::D3D12_TEXTURE_LAYOUT_UNKNOWN,
            Flags: flags,
            Format: format,
            ..mem::zeroed()
        };

        let buffer = self.create_committed_resource(
            &heap_properties,
            //TODO: is this heap flag ok?
            d3d12::D3D12_HEAP_FLAG_NONE,
            &resource_description,
            initial_resource_state,
            ptr::null(),
            descriptor_heap_offset,
        );

        buffer
    }

    pub unsafe fn get_removal_reason(&self) -> String {
        error::convert_hresult_to_lower_hex(self.0.GetDeviceRemovedReason())
    }
}

pub struct SubresourceData {
    pub data: Vec<u8>,
    pub row_size: isize,
    pub column_size: isize,
}

impl SubresourceData {
    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn as_d3d12_subresource_data(&self) -> d3d12::D3D12_SUBRESOURCE_DATA {
        assert_eq!(self.row_size % 256, 0);

        d3d12::D3D12_SUBRESOURCE_DATA {
            pData: self.data.as_ptr() as *const _,
            RowPitch: self.row_size,
            SlicePitch: self.column_size,
        }
    }
}

impl CommandAllocator {
    pub unsafe fn reset(&self) {
        self.0.Reset();
    }
}

impl DescriptorHeap {
    unsafe fn get_cpu_descriptor_handle_for_heap_start(&self) -> CpuDescriptor {
        self.heap.GetCPUDescriptorHandleForHeapStart()
    }

    unsafe fn get_gpu_descriptor_handle_for_heap_start(&self) -> GpuDescriptor {
        self.heap.GetGPUDescriptorHandleForHeapStart()
    }

    pub unsafe fn get_cpu_descriptor_handle_at_offset(&self, offset: u32) -> CpuDescriptor {
        let mut descriptor = self.get_cpu_descriptor_handle_for_heap_start();
        descriptor.ptr += (offset as usize) * (self.increment_size as usize);

        descriptor
    }

    pub unsafe fn get_gpu_descriptor_handle_at_offset(&self, offset: u32) -> GpuDescriptor {
        let mut descriptor = self.get_gpu_descriptor_handle_for_heap_start();
        descriptor.ptr += (offset as u64) * (self.increment_size as u64);

        descriptor
    }
}

#[repr(transparent)]
pub struct DescriptorRange(d3d12::D3D12_DESCRIPTOR_RANGE);
impl DescriptorRange {}

impl RootSignature {
    pub unsafe fn serialize_description(
        desc: &d3d12::D3D12_ROOT_SIGNATURE_DESC,
        version: d3d12::D3D_ROOT_SIGNATURE_VERSION,
    ) -> Blob {
        let mut blob = ptr::null_mut();
        //TODO: properly use error blob
        let mut _error = ptr::null_mut();

        error::error_if_failed_else_unit(d3d12::D3D12SerializeRootSignature(
            desc as *const _,
            version,
            &mut blob as *mut _ as *mut _,
            &mut _error as *mut _ as *mut _,
        ))
        .expect("could not serialize root signature description");

        Blob(ComPtr::from_raw(blob))
    }
}

impl ShaderByteCode {
    // empty byte code
    pub unsafe fn empty() -> ShaderByteCode {
        ShaderByteCode {
            bytecode: d3d12::D3D12_SHADER_BYTECODE {
                BytecodeLength: 0,
                pShaderBytecode: ptr::null(),
            },
            blob: None,
        }
    }

    // `blob` may not be null.
    pub unsafe fn from_blob(blob: Blob) -> ShaderByteCode {
        ShaderByteCode {
            bytecode: d3d12::D3D12_SHADER_BYTECODE {
                BytecodeLength: blob.0.GetBufferSize(),
                pShaderBytecode: blob.0.GetBufferPointer(),
            },
            blob: Some(blob),
        }
    }

    /// Compile a shader from raw HLSL.
    ///
    /// * `target`: example format: `ps_5_1`.
    pub unsafe fn compile(
        source: String,
        target: String,
        entry: String,
        flags: minwindef::DWORD,
    ) -> Blob {
        let mut shader_blob_ptr: *mut ID3DBlob = ptr::null_mut();
        //TODO: use error blob properly
        let mut error_blob_ptr: *mut ID3DBlob = ptr::null_mut();

        let target = ffi::CString::new(target)
            .expect("could not convert target format string into ffi::CString");
        let entry = ffi::CString::new(entry)
            .expect("could not convert entry name String into ffi::CString");

        let hresult = d3dcompiler::D3DCompile(
            source.as_ptr() as *const _,
            source.len(),
            ptr::null(),
            ptr::null(),
            d3dcompiler::D3D_COMPILE_STANDARD_FILE_INCLUDE,
            entry.as_ptr() as *const _,
            target.as_ptr() as *const _,
            flags,
            0,
            &mut shader_blob_ptr as *mut _ as *mut _,
            &mut error_blob_ptr as *mut _ as *mut _,
        );

        #[cfg(debug_assertions)]
        {
            if !error_blob_ptr.is_null() {
                let error_blob = Blob(ComPtr::from_raw(error_blob_ptr));
                Blob::print_to_console(error_blob.clone());
            }
        }

        error::error_if_failed_else_unit(hresult).expect("shader compilation failed");

        Blob(ComPtr::from_raw(shader_blob_ptr))
    }

    pub unsafe fn compile_from_file(
        file_path: &Path,
        target: String,
        entry: String,
        flags: minwindef::DWORD,
    ) -> Blob {
        let file_open_error = format!("could not open shader source file for entry: {}", &entry);
        let source = std::fs::read_to_string(file_path).expect(&file_open_error);

        ShaderByteCode::compile(source, target, entry, flags)
    }
}

impl Fence {
    pub unsafe fn set_event_on_completion(&self, event: Event, value: u64) -> winerror::HRESULT {
        self.0.SetEventOnCompletion(value, event.0)
    }

    pub unsafe fn get_value(&self) -> u64 {
        self.0.GetCompletedValue()
    }

    pub unsafe fn signal(&self, value: u64) -> winerror::HRESULT {
        self.0.Signal(value)
    }
}

impl Event {
    pub unsafe fn create(manual_reset: bool, initial_state: bool) -> Self {
        Event(synchapi::CreateEventA(
            ptr::null_mut(),
            manual_reset as _,
            initial_state as _,
            ptr::null(),
        ))
    }

    pub unsafe fn wait(&self, timeout_ms: u32) -> u32 {
        synchapi::WaitForSingleObject(self.0, timeout_ms)
    }

    pub unsafe fn wait_ex(&self, timeout_ms: u32, alertable: bool) -> u32 {
        synchapi::WaitForSingleObjectEx(self.0, timeout_ms, alertable as _)
    }
}

impl GraphicsCommandList {
    pub unsafe fn as_raw_list(&self) -> *mut d3d12::ID3D12CommandList {
        self.0.as_raw() as *mut d3d12::ID3D12CommandList
    }

    pub unsafe fn close(&self) -> winerror::HRESULT {
        self.0.Close()
    }

    pub unsafe fn reset(&self, allocator: CommandAllocator, initial_pso: PipelineState) {
        error::error_if_failed_else_unit(
            self.0.Reset(allocator.0.as_raw(), initial_pso.0.as_raw()),
        )
        .expect("could not reset command list");
    }

    pub unsafe fn set_compute_pipeline_root_signature(&self, signature: RootSignature) {
        self.0.SetComputeRootSignature(signature.0.as_raw());
    }

    pub unsafe fn set_graphics_pipeline_root_signature(&self, signature: RootSignature) {
        self.0.SetGraphicsRootSignature(signature.0.as_raw());
    }

    pub unsafe fn set_resource_barrier(
        &self,
        resource_barriers: Vec<d3d12::D3D12_RESOURCE_BARRIER>,
    ) {
        self.0.ResourceBarrier(
            u32::try_from(resource_barriers.len())
                .expect("could not safely convert resource_barriers.len() into u32"),
            (&resource_barriers).as_ptr(),
        );
    }

    pub unsafe fn set_viewport(&self, viewport: &d3d12::D3D12_VIEWPORT) {
        self.0.RSSetViewports(1, viewport as *const _);
    }

    pub unsafe fn set_scissor_rect(&self, scissor_rect: &d3d12::D3D12_RECT) {
        self.0.RSSetScissorRects(1, scissor_rect as *const _);
    }

    pub unsafe fn dispatch(&self, count_x: u32, count_y: u32, count_z: u32) {
        self.0.Dispatch(count_x, count_y, count_z);
    }

    pub unsafe fn draw_instanced(
        &self,
        num_vertices: u32,
        num_instances: u32,
        start_vertex: u32,
        start_instance: u32,
    ) {
        self.0
            .DrawInstanced(num_vertices, num_instances, start_vertex, start_instance);
    }

    pub unsafe fn set_pipeline_state(&self, pipeline_state: PipelineState) {
        self.0.SetPipelineState(pipeline_state.0.as_raw());
    }

    pub unsafe fn set_compute_root_unordered_access_view(
        &self,
        root_parameter_index: u32,
        buffer_location: d3d12::D3D12_GPU_VIRTUAL_ADDRESS,
    ) {
        self.0
            .SetComputeRootUnorderedAccessView(root_parameter_index, buffer_location);
    }

    pub unsafe fn set_compute_root_descriptor_table(
        &self,
        root_parameter_index: u32,
        base_descriptor: d3d12::D3D12_GPU_DESCRIPTOR_HANDLE,
    ) {
        self.0
            .SetComputeRootDescriptorTable(root_parameter_index, base_descriptor);
    }

    pub unsafe fn set_graphics_root_shader_resource_view(
        &self,
        root_parameter_index: u32,
        buffer_location: d3d12::D3D12_GPU_VIRTUAL_ADDRESS,
    ) {
        self.0
            .SetGraphicsRootShaderResourceView(root_parameter_index, buffer_location);
    }

    pub unsafe fn set_graphics_root_descriptor_table(
        &self,
        root_parameter_index: u32,
        base_descriptor: d3d12::D3D12_GPU_DESCRIPTOR_HANDLE,
    ) {
        self.0
            .SetGraphicsRootDescriptorTable(root_parameter_index, base_descriptor);
    }

    pub unsafe fn set_render_target(
        &self,
        render_target_descriptor: d3d12::D3D12_CPU_DESCRIPTOR_HANDLE,
    ) {
        self.0.OMSetRenderTargets(
            1,
            &render_target_descriptor as *const _,
            false as _,
            ptr::null(),
        );
    }

    pub unsafe fn clear_render_target_view(
        &self,
        render_target_descriptor: d3d12::D3D12_CPU_DESCRIPTOR_HANDLE,
        clear_color: &[f32; 4],
    ) {
        self.0.ClearRenderTargetView(
            render_target_descriptor,
            clear_color as *const _,
            0,
            ptr::null(),
        );
    }

    pub unsafe fn set_primitive_topology(
        &self,
        primitive_topology: d3dcommon::D3D_PRIMITIVE_TOPOLOGY,
    ) {
        self.0.IASetPrimitiveTopology(primitive_topology);
    }

    pub unsafe fn set_vertex_buffer(
        &self,
        start_slot: u32,
        num_views: u32,
        vertex_buffer_view: &d3d12::D3D12_VERTEX_BUFFER_VIEW,
    ) {
        self.0
            .IASetVertexBuffers(start_slot, num_views, vertex_buffer_view as *const _);
    }

    pub unsafe fn set_descriptor_heaps(&self, descriptor_heaps: Vec<DescriptorHeap>) {
        let descriptor_heap_pointers: Vec<*mut d3d12::ID3D12DescriptorHeap> =
            descriptor_heaps.iter().map(|dh| dh.heap.as_raw()).collect();
        self.0.SetDescriptorHeaps(
            u32::try_from(descriptor_heap_pointers.len())
                .expect("could not safely convert descriptor_heap_pointers.len() into u32"),
            (&descriptor_heap_pointers).as_ptr() as *mut _,
        );
    }

    pub unsafe fn end_timing_query(&self, query_heap: QueryHeap, index: u32) {
        self.0.EndQuery(
            query_heap.0.as_raw() as *mut _,
            d3d12::D3D12_QUERY_TYPE_TIMESTAMP,
            index,
        );
    }

    pub unsafe fn resolve_timing_query_data(
        &self,
        query_heap: QueryHeap,
        start_index: u32,
        num_queries: u32,
        destination_buffer: Resource,
        aligned_destination_buffer_offset: u64,
    ) {
        self.0.ResolveQueryData(
            query_heap.0.as_raw() as *mut _,
            d3d12::D3D12_QUERY_TYPE_TIMESTAMP,
            start_index,
            num_queries,
            destination_buffer.com_ptr.as_raw() as *mut _,
            aligned_destination_buffer_offset,
        );
    }
    pub unsafe fn update_texture2d_using_intermediate_buffer(
        &self,
        device: Device,
        intermediate_buffer: Resource,
        texture: Resource,
    ) {
        let mut src = d3d12::D3D12_TEXTURE_COPY_LOCATION {
            pResource: intermediate_buffer.com_ptr.as_raw(),
            Type: d3d12::D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
            ..mem::zeroed()
        };
        let (layout, _, _, _) = device.get_copyable_footprint(0, 1, 0, texture.clone());
        *src.u.PlacedFootprint_mut() = layout[0];

        let mut dst = d3d12::D3D12_TEXTURE_COPY_LOCATION {
            pResource: texture.com_ptr.as_raw(),
            Type: d3d12::D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
            ..mem::zeroed()
        };
        *dst.u.SubresourceIndex_mut() = 0;

        self.0
            .CopyTextureRegion(&dst as *const _, 0, 0, 0, &src as *const _, ptr::null());
    }
}

pub fn default_render_target_blend_desc() -> d3d12::D3D12_RENDER_TARGET_BLEND_DESC {
    d3d12::D3D12_RENDER_TARGET_BLEND_DESC {
        BlendEnable: minwindef::FALSE,
        LogicOpEnable: minwindef::FALSE,
        SrcBlend: d3d12::D3D12_BLEND_ONE,
        DestBlend: d3d12::D3D12_BLEND_ZERO,
        // enum variant 0
        BlendOp: d3d12::D3D12_BLEND_OP_ADD,
        SrcBlendAlpha: d3d12::D3D12_BLEND_ONE,
        DestBlendAlpha: d3d12::D3D12_BLEND_ZERO,
        BlendOpAlpha: d3d12::D3D12_BLEND_OP_ADD,
        // enum variant 0
        LogicOp: d3d12::D3D12_LOGIC_OP_NOOP,
        RenderTargetWriteMask: d3d12::D3D12_COLOR_WRITE_ENABLE_ALL as u8,
    }
}

pub fn default_blend_desc() -> d3d12::D3D12_BLEND_DESC {
    // see default description here: https://docs.microsoft.com/en-us/windows/win32/direct3d12/cd3dx12-blend-desc
    d3d12::D3D12_BLEND_DESC {
        AlphaToCoverageEnable: minwindef::FALSE,
        IndependentBlendEnable: minwindef::FALSE,
        RenderTarget: [
            default_render_target_blend_desc(),
            default_render_target_blend_desc(),
            default_render_target_blend_desc(),
            default_render_target_blend_desc(),
            default_render_target_blend_desc(),
            default_render_target_blend_desc(),
            default_render_target_blend_desc(),
            default_render_target_blend_desc(),
        ],
    }
}

pub unsafe fn create_uav_resource_barrier(
    resource: *mut d3d12::ID3D12Resource,
) -> d3d12::D3D12_RESOURCE_BARRIER {
    let uav = d3d12::D3D12_RESOURCE_UAV_BARRIER {
        pResource: resource,
    };

    let mut resource_barrier: d3d12::D3D12_RESOURCE_BARRIER = mem::zeroed();
    resource_barrier.Type = d3d12::D3D12_RESOURCE_BARRIER_TYPE_UAV;
    resource_barrier.Flags = d3d12::D3D12_RESOURCE_BARRIER_FLAG_NONE;
    *resource_barrier.u.UAV_mut() = uav;

    resource_barrier
}

pub unsafe fn create_transition_resource_barrier(
    resource: *mut d3d12::ID3D12Resource,
    state_before: d3d12::D3D12_RESOURCE_STATES,
    state_after: d3d12::D3D12_RESOURCE_STATES,
) -> d3d12::D3D12_RESOURCE_BARRIER {
    let transition = d3d12::D3D12_RESOURCE_TRANSITION_BARRIER {
        pResource: resource,
        Subresource: d3d12::D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
        StateBefore: state_before,
        StateAfter: state_after,
    };

    let mut resource_barrier: d3d12::D3D12_RESOURCE_BARRIER = mem::zeroed();
    resource_barrier.Type = d3d12::D3D12_RESOURCE_BARRIER_TYPE_TRANSITION;
    resource_barrier.Flags = d3d12::D3D12_RESOURCE_BARRIER_FLAG_NONE;
    *resource_barrier.u.Transition_mut() = transition;

    resource_barrier
}

pub unsafe fn enable_debug_layer() {
    println!("enabling debug layer.");

    let mut debug_controller: *mut d3d12sdklayers::ID3D12Debug1 = ptr::null_mut();
    error::error_if_failed_else_unit(d3d12::D3D12GetDebugInterface(
        &d3d12sdklayers::ID3D12Debug1::uuidof(),
        &mut debug_controller as *mut _ as *mut _,
    ))
    .expect("could not create debug controller");

    (*debug_controller).EnableDebugLayer();

    let mut queue = ptr::null_mut();
    let hr = dxgi1_3::DXGIGetDebugInterface1(
        0,
        &dxgidebug::IDXGIInfoQueue::uuidof(),
        &mut queue as *mut _ as *mut _,
    );

    if winerror::SUCCEEDED(hr) {
        (*debug_controller).SetEnableGPUBasedValidation(minwindef::TRUE);
    } else {
        println!("failed to enable debug layer!");
    }

    (*debug_controller).Release();
}

pub struct InputElementDesc {
    pub semantic_name: String,
    pub semantic_index: u32,
    pub format: dxgiformat::DXGI_FORMAT,
    pub input_slot: u32,
    pub aligned_byte_offset: u32,
    pub input_slot_class: d3d12::D3D12_INPUT_CLASSIFICATION,
    pub instance_data_step_rate: u32,
}

impl InputElementDesc {
    pub fn as_winapi_struct(&self) -> d3d12::D3D12_INPUT_ELEMENT_DESC {
        d3d12::D3D12_INPUT_ELEMENT_DESC {
            SemanticName: std::ffi::CString::new(self.semantic_name.as_str())
                .unwrap()
                .into_raw() as *const _,
            SemanticIndex: self.semantic_index,
            Format: self.format,
            InputSlot: self.input_slot,
            AlignedByteOffset: self.aligned_byte_offset,
            InputSlotClass: self.input_slot_class,
            InstanceDataStepRate: self.instance_data_step_rate,
        }
    }
}
